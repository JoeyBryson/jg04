# Synchronization Protocol: Feasibility and Change Report

## 1. Verdict

The idea is feasible and fits the existing architecture well. A `SynchronizationProtocol` can be added next to `ControlProtocol` as another iroh `ProtocolHandler` on its own ALPN. It would also give you two things you don't have today: catch-up after being offline, and history for newly joined members.

The hash-chain design as described has four problems that need decisions before coding:

1. Gossip delivers messages in different orders to different peers, so a "hash of previous hash plus new message" chain only matches if you define a canonical order. That means rehashing the tail whenever a message arrives out of order.
2. The current message format cannot support sync. It does not store author signatures, and it identifies the author using the wrong field (see 3.1). This must be fixed first.
3. Two peers can append messages concurrently, so "same last hash means same messages" only holds once ordering is canonical.
4. "Last N hashes" has a failure mode when the divergence is larger than N. An exponentially spaced sample avoids it.

**Decision: Option A, a single global ledger with canonical ordering, is selected.** Its tail-rehash cost appears acceptable for ordinary group chats with a few hundred messages; possible public-forum scale should be capped and benchmarked. Option B remains an alternative if measured upper-bound write latency is unacceptable.

## 2. What exists today

| Area | Current state | Relevance |
|---|---|---|
| Protocol wiring | [service.rs](rust-core/src/network/application/service.rs) registers `CONTROL_ALPN` and `iroh_gossip::ALPN` on the `Endpoint` and `Router`. | A sync ALPN is added in the same two places. |
| Control protocol | [handler.rs](rust-core/src/network/control/handler.rs) handles one request and one response per stream, using `read_to_end(1 MiB)`. | Sync needs multiple rounds, so it needs framing instead. |
| Gossip receive | [session.rs](rust-core/src/network/chat/session.rs) `receive_loop` ignores `NeighborUp`/`NeighborDown`. | `NeighborUp` is the natural trigger for sync. |
| Wire format | [signed.rs](rust-core/src/network/messaging/signed.rs) signs `MessageData { content, sent_at }` only. | No topic, no author, no kind. |
| Message storage | `messages(id AUTOINCREMENT, topic_id, endpoint_id, content, sent_at)` in [schema.sql](rust-core/src/database/sql/schema.sql). | No signature, no message id, no uniqueness. |
| Ordering | `nw_reads` orders by `id` (arrival order). `ui_reads` orders by `sent_at, id`. | Arrival order differs per peer, so it is unusable for a chain. |
| DB access | A single `DbWriter` thread serializes all writes; requests are declared in [macros.rs](rust-core/src/database/macros.rs) `db_requests!`. | Chain updates are race-free if they all go through that writer. |
| Migrations | None. `schema.sql` is `CREATE IF NOT EXISTS`, and the note in [mod.rs](rust-core/src/database/mod.rs) says to reset the db after schema changes. | A schema change is acceptable pre-release. |
| Dependencies | `blake3` and `sha2` are already in `Cargo.lock` (transitive). | Add one as a direct dependency. |

## 3. Existing issues that block this feature

### 3.1 Author is taken from `delivered_from`, which is wrong

`verify_and_decode` uses `gossip_message.delivered_from` as both the sender and the signature verification key. iroh-gossip documents this field as: "The endpoint that delivered the message. This is not the same as the original author."

Consequences today:

- A message relayed through a third peer fails signature verification and is dropped as "invalid gossip message". This is hidden in small groups where everyone is a direct neighbor.
- For sync it is fatal. A peer sending you messages from other authors cannot be identified by connection identity, so each message must carry its own author and signature, and you must store them.

### 3.2 Signatures are verified and then discarded

Only `content`, `sender` and `sent_at` are stored. Without stored signatures, a syncing peer could forge messages "from" anyone, and you could not re-serve verifiable messages to a third peer.

### 3.3 The signed bytes do not bind the chat

The signature covers only `content` and `sent_at`. A message can be replayed into another chat. Include `topic_id` in the signed payload.

### 3.4 No deduplication

There is no unique key on `messages`. Once messages arrive by both gossip and sync, duplicates are guaranteed unless a message id is added with a unique constraint.

## 4. Required data model changes

### 4.1 Message identity

```text
SignedMessage {
    author:     EndpointId,
    topic_id:   TopicId,
    sent_at:    u64,
    kind:       Chat { content } | (future) Delete { target_id },
    signature:  Signature,        // over (author, topic_id, sent_at, kind)
}
message_id = BLAKE3(canonical postcard bytes of the signed payload)
```

Notes:

- postcard encoding of a fixed struct is deterministic, so it is usable as a hash input.
- The chain hash should be built from `message_id`, not from the raw content (see 4.3). This is what makes deletes workable later.
- Two identical messages from the same author in the same millisecond collapse to one. That is acceptable.

### 4.2 Gossip envelope

Gossip must carry more than chat messages once heads are broadcast:

```text
GossipPayload::Chat(SignedMessage)
GossipPayload::Head { chain_head: [u8; 32], count: u64 }
```

This changes the gossip wire format. There are no real users yet, so that is cheap now and expensive later.

### 4.3 Schema (Option A, linear chain)

```sql
ALTER messages ADD:
    message_id  BLOB NOT NULL,   -- BLAKE3 of signed payload
    signature   BLOB NOT NULL,
    seq         INTEGER NOT NULL, -- position in canonical order, 0-based
    chain_hash  BLOB NOT NULL,   -- BLAKE3(prev_chain_hash || message_id)
    UNIQUE (topic_id, message_id)

INDEX messages_order ON messages (topic_id, sent_at, endpoint_id, message_id)
```

Because the schema is `CREATE IF NOT EXISTS` with no migrations, the practical route is to edit the `CREATE TABLE` and reset the dev db.

The current product model requires every author to be in `contacts`. Sync should therefore reject messages whose author is not a contact, and the schema should enforce `messages.endpoint_id -> contacts(endpoint_id)` with a foreign key. The existing diagram already implies this relationship, but the SQL currently omits it. Ensure the per-connection SQLite foreign-key setting is enabled (see the assumption report) or the constraint will not protect writes.

### 4.4 Canonical order (Option A)

For the chain to be identical across peers, every peer must sort the same set the same way:

```text
ORDER BY sent_at ASC, endpoint_id ASC, message_id ASC
```

Consequences:

- `nw_reads` currently orders by `id`. It must switch to this order, and `ui_reads` should match it.
- A message arriving with a key lower than the current tail forces recomputation of `seq` and `chain_hash` for every message after it, in one transaction.
- Out-of-order arrival is not rare. It happens routinely with gossip, with clock skew between devices, and always during sync.
- In the common case the insert is at the tail, so the cost is O(1). A message inserted deep in history costs O(n - p). With BLAKE3 the hashing is trivial and the `UPDATE`s dominate. For ordinary chats with a few hundred messages this is expected to be small. The work runs on the single `DbWriter` thread and delays other writes, so benchmark near the intended history cap before expanding to a large public forum. Sync must batch inserts and rehash once from the earliest affected position.
- `sent_at` is author-controlled. Any member can place a message at an arbitrary position in history, and the UI will show it there. A server-free chat cannot fully prevent this. Decide whether to accept it or clamp (for example, reject `sent_at` too far in the future).

## 5. Alternatives to the single linear chain

### Option A: selected — one global ledger with canonical ordering

- Pros: simple mental model, one hash per chat, most direct to what you described; any joined peer can reconcile the shared ledger rather than relying on each original author being available.
- Cons: tail rehash on out-of-order insert, `sent_at` is trust-based, fork-finding needs the sampling scheme in 6.3.

### Option B: per-author append-only logs (Secure Scuttlebutt style)

Each message carries `author_seq` and `prev_author_hash`, so each author has its own hash chain.

```text
sync state = { author -> (latest_seq, latest_hash) }
chat head  = BLAKE3(sorted entries of that map)       // still a single 32-byte value to gossip
```

- No canonical global order is needed for sync, so no rehashing, ever. Insertion is always an append to an author's log.
- Sync state is small: a sorted map from author to `(latest_seq, latest_hash)`. For each author, peers exchange the suffix after the other side's sequence; the first received entry must point to the receiver's current hash.
- No "last N" window and no binary search.
- Deletes become another entry in the author's log.
- Display order stays `sent_at, id` as now.
- Cons:
  - **Gaps:** if seq 8 arrives but seq 7 is missing, discard 8 and request the missing suffix directly from the original author named in the signed message. `delivered_from` is only the gossip relay and must not select the sync target. The author serves its contiguous log from seq 7, and the receiver verifies signatures and predecessor hashes before advancing its head. Persist the missing-sequence request and retry when that author reconnects.
  - **Availability limit:** direct-author repair depends on the original device being reachable and retaining its log. While it is offline or unreachable, the gap remains and later messages from that author cannot enter the verified contiguous chain. This gives eventual repair when the author returns, not delivery despite permanent author loss. A future replica fallback could fetch another peer's copy while still verifying the original author's signature and chain links, but that is not the v1 policy.
  - Prevent permanent gaps caused by the author itself by allocating the sequence and persisting the signed message before gossip publication, atomically on the sending device.
  - **Forks:** two writers using the same author key can independently create different messages at the same sequence number. Both signatures can be valid, but their hashes diverge and there is no single next hash. This can happen with a restored/cloned backup or multiple devices sharing one identity. The current profile model has one key per installation, which lowers the risk but does not prevent key copies. You must choose a policy: reject and surface the conflict, keep both branches, or introduce distinct per-device author IDs/keys. Silently choosing one branch would discard valid signed data.
  - It needs `author_seq` and `prev_author_hash` in the signed payload. Sequence allocation must be serialized and durable before publishing. A second device sharing the key cannot safely coordinate this without additional protocol/state.
  - There is no single cross-author order in the chain. The UI can still sort by `sent_at, author, message_id`, but that is a presentation order, not a causal total order.

These are engineering costs, not reasons Option B cannot work. If the product guarantees one active writer per author key and the database durably appends before broadcasting, the gap/fork risks are contained: gaps are temporary and sync fills them; forks should be impossible except after key cloning or restore. If multi-device-per-identity is a goal, model each device as its own author stream rather than sharing one sequence.

Option B is not selected. Its key benefit is eliminating global rehashing; its trade-offs are per-author gap/fork state and the lack of a single causal total order. Reconsider it if the single-ledger benchmark or forum scale makes tail rehashing unacceptable.

### Option C: set reconciliation (range-based, as used by iroh-docs / Negentropy)

No chain at all: peers compare fingerprints of ranges of message ids and recurse into ranges that differ. It is robust and handles arbitrary divergence, but it is the most complex to implement. Worth knowing about, probably overkill here.

Section 6.3 gives a handshake for Option A. For Option B, use an author-targeted request containing the topic, original author, first missing sequence, and expected predecessor hash. The responder must be that author, verify the requester is a current joined member, and serve only its own contiguous signed suffix. The receiver verifies topic, author, sequence, signature, and hash links before committing. Keep the gap request durable and retry it on that author's reconnect. Other framing and size limits still apply.

## 6. Protocol design

### 6.1 Triggers

1. On a sequence gap, connect directly to the original author in the signed message and request the missing suffix. Use `NeighborUp(author)` to retry pending requests when that original author reconnects.
2. Start with gap-triggered requests and reconnect retries. Optional signed per-author head gossip can reduce detection delay later, but should not substitute a relay/other peer for the original author under the v1 policy.

Pitfalls with head broadcast:

- Head gossip is O(members) messages per interval per member, so keep the interval long and the payload small.
- A message in flight makes heads differ momentarily. Debounce: only start a sync if the peer's head differs on two consecutive announcements or remains different after a short delay.
- Both peers may notice the difference simultaneously. Tie-break: the peer with the lower `EndpointId` initiates; the other ignores it.
- Rate-limit syncs per `(topic, peer)` and allow at most one in flight.
- `Head` is unauthenticated unless signed. If unsigned, an attacker can only cause extra syncs, so rate limiting covers it. Signing it is cheap and still worth doing.

### 6.2 Authorization

`ProtocolHandler::accept` gives you the authenticated remote via `Connection::remote_id()`. The handler must check that the remote is a `Joined` member of the requested topic before serving anything. The topic id is a random 32 bytes, so topic knowledge alone should not grant access. `ControlProtocol` currently does no check of this kind for invites; that is a separate matter (see [chat-invite-system-audit.md](chat-invite-system-audit.md)).

Decision to make: should a newly invited member receive the full history? With this design they will.

### 6.3 Handshake (Option A)

Run a multi-round exchange over a single bidirectional stream with length-prefixed postcard frames (not `read_to_end`).

```text
Initiator                                  Responder
Hello { topic, my_head, my_count }   ->
                                     <-    Hello { topic, head, count }      (equal -> Done)
Samples { [(seq, chain_hash)...] }   ->
                                     <-    Samples { ... }
   both compute fork = highest seq where hashes agree (or "none")
HaveIds { ids after fork }           ->
                                     <-    HaveIds { ids after fork }
Messages { missing ones }            ->
                                     <-    Messages { missing ones }
   both verify signatures, insert in one transaction, recompute chain
Final { head, count }                ->
                                     <-    Final { head, count }
   equal -> Done; not equal -> repeat once (bounded), then give up
```

Sampling instead of "last N":

- Send hashes at exponentially spaced offsets from the tail (1, 2, 4, 8, ... positions back) plus the first position. That is about log2(n) entries, a few hundred bytes for any chat size.
- This is valid because chain hash equality at a position implies identical prefix, so "agrees at k" is monotone and the fork point is found by a few narrowing rounds. A fixed window of N breaks when the divergence exceeds N and would need a fallback anyway.
- If you do keep a fixed N, the data cost is irrelevant (100 hashes is about 3 KB). Only the fallback matters.

"Send all after fork" versus "send only what's missing":

- After the fork, both sides may hold different, interleaved messages, so each side's tail is a mix of shared and unique messages.
- Exchanging ids first (32 bytes each) and then only the missing messages is the efficient option. It costs one extra round trip.
- Simplest v1: send full messages after the fork and rely on `UNIQUE(topic_id, message_id)` to dedupe. This wastes bandwidth only proportional to the divergence. I would start with this and add the id exchange later.

### 6.4 Applying received messages

For each received message:

1. Verify the signature against `author` inside the message (not the connection identity).
2. Check `topic_id` matches the chat.
3. Check `sent_at` is within a sane window.
4. Insert with `ON CONFLICT DO NOTHING`.

Then recompute `seq` and `chain_hash` from the earliest inserted position to the tail, in the same transaction as the inserts. Peers should do the whole batch atomically; a dropped connection mid-sync then leaves no partial state, and a retry is safe because everything is idempotent.

### 6.5 What "guaranteed" can and cannot mean

- Guaranteed: at the end of a successful session, both peers hold the union of both sets as of the snapshot and have verified equal heads via the `Final` exchange.
- Not guaranteed: that messages arriving during the sync are included. They trigger another head mismatch and another sync. The system is eventually consistent, not transactionally consistent.
- Because each side applies in a single transaction and the `Final` frame is sent only after commit, each peer knows the other has committed when it receives a matching `Final`.

### 6.6 Limits

- Cap frame size (for example 256 KiB) and message count per frame, chunking larger transfers.
- Cap total transfer per session and total session time (the invite code uses a 5 second timeout; sync needs longer and per-frame timeouts).
- Cap `Samples` length.
- Reject unknown topics and non-members immediately.

## 7. Code structure and changes

### 7.1 New module, mirroring `control/`

```text
rust-core/src/network/sync/
    mod.rs          // re-exports
    messages.rs     // SyncRequest/SyncFrame enums (postcard)
    framing.rs      // length-prefixed read/write helpers, size caps
    handler.rs      // SyncProtocol: ProtocolHandler (responder side)
    initiator.rs    // SyncHandle actor: SyncWith { topic, peer }, rate limiting, dedupe
```

### 7.2 Wiring in existing files

| File | Change |
|---|---|
| [network/mod.rs](rust-core/src/network/mod.rs) | Add `SYNC_ALPN` (for example `b"jg04/sync/1"`) and `mod sync`. Consider also renaming the leftover `iroh-example/echo/0` control ALPN. |
| [service.rs](rust-core/src/network/application/service.rs) | Add `SYNC_ALPN` to `.alpns(...)`, `.accept(SYNC_ALPN, SyncProtocol { .. })` on the `Router`, and spawn the initiator actor. |
| [session.rs](rust-core/src/network/chat/session.rs) | Handle `NeighborUp` (request sync) and the new `Head` payload; spawn a periodic head broadcaster; `send` stores the signed message, not just the content. |
| [sessions.rs](rust-core/src/network/chat/sessions.rs) | Pass the `SyncHandle` into `ChatSession::spawn`. |
| [signed.rs](rust-core/src/network/messaging/signed.rs) | New `SignedMessage` carrying author, topic, kind and signature; verify with the embedded author; the `GossipPayload` envelope. |
| [persistence/sessions.rs](rust-core/src/network/persistence/sessions.rs) | `record_message` takes the signed message. Add a `SyncStore` for chain reads and batched writes. |

Construction order issue: `ChatInvitesHandle` takes the `Router`, which is created after `ChatSessionsHandle` because `ControlProtocol` needs `chat_sessions`. The sync initiator should take an `Endpoint` clone (it only needs `connect`), so it can be created before the sessions and passed in without a cycle. `SyncProtocol` needs only DB access, not the sessions, so it has no cycle either.

### 7.3 Database

- [schema.sql](rust-core/src/database/sql/schema.sql): new columns, `UNIQUE`, index, comment updates (section 4.3).
- [writes.rs](rust-core/src/database/workers/writes.rs): replace `add_nw_message` with an insert-and-rechain function that runs in a transaction; add a batch variant for sync. It should return the earliest position changed.
- [nw_reads.rs](rust-core/src/database/workers/nw_reads.rs): `GetChatHead`, `GetChainSamples(topic, offsets)`, `GetMessagesAfter(topic, seq)`, `GetMessageIds(topic, after_seq)`; change message ordering to the canonical order.
- [ui_reads.rs](rust-core/src/database/workers/ui_reads.rs): align ordering with the canonical order.
- [macros.rs](rust-core/src/database/macros.rs): register the new requests in `db_requests!`. Batch inserts should emit one `ChatDataChanged` and one `ChatHeadersChanged`, not one per message.
- All of this stays behind the single writer thread, which is what makes the chain update race-free.

### 7.4 Dependencies

Add `blake3` as a direct dependency (it is already in the dependency tree).

### 7.5 FFI and UI

No UniFFI surface change is strictly needed: the UI already reacts to `ChatDataChanged`. Optional: a "syncing" indicator event. Note that messages can now appear in the middle of the visible history when sync inserts older messages; the UI should handle insertions, not only appends.

## 8. Deletes (future, for awareness only)

A design choice now makes this easy later:

- Hash `message_id` into the chain, not the raw content. A delete is then just another signed message `Delete { target_id }` with its own chain position.
- The target's content can be purged while its `message_id` row (tombstone) stays, so chain hashes remain valid.
- A delete can arrive before its target (out of order or through sync). Keep an "applied deletes" check so that a later-arriving target is stored as already deleted.
- Only the original author's `Delete` should be honored.

## 9. Testing

- DB tests, following [writes_test.rs](rust-core/src/database/workers/writes_test.rs):
  - The same set inserted in any order gives identical heads (a permutation test; the key correctness property for Option A).
  - Duplicate insert is a no-op.
  - Out-of-order insert rewrites `seq` and `chain_hash` correctly.
- Unit tests for the sampling and fork-point computation, including divergence larger than the sample size and an empty chain on one side.
- Network tests, extending [tests.rs](rust-core/src/network/tests.rs) (which already uses the iroh test relay):
  - Two peers with disjoint messages converge to equal heads.
  - Offline peer catches up after reconnect.
  - A newly invited member receives history.
  - Non-member connection is rejected.
  - Forged author, wrong topic, or oversized frame is rejected.
  - Connection dropped mid-sync leaves no partial state and the retry succeeds.

## 10. Suggested order of work

1. Fix the message format: embedded author, topic in signature, kind, stored signature, `message_id`, unique constraint, and versioned `GossipPayload`. This is needed whichever reconciliation strategy is used and fixes the `delivered_from` bug.
2. Implement the selected global ledger: canonical `(sent_at, endpoint_id, message_id)` order, incremental/batch rehashing, and permutation tests proving identical heads for the same message set.
3. Benchmark tail, middle, and beginning inserts at representative sizes and near the proposed maximum. Define cap behavior explicitly; don't silently prune history.
4. Add the `SyncProtocol` responder and initiator with framing, membership authorization, limits, and sampled fork discovery.
5. Trigger sync on `NeighborUp`; add debounced signed head gossip only if testing shows it is needed.
6. Add ID-exchange optimization and checkpoints only if measurements justify them.

## 11. Open decisions

| Question | Options | Suggestion |
|---|---|---|
| Chain structure | A: global linear chain, B: per-author logs | B unless you need one global total order |
| Canonical order (A) | `sent_at`, then author, then id; or Lamport timestamp | `sent_at, author, id`, with a clamp on future timestamps |
| Fork search | last N, or exponential sampling | Exponential sampling |
| Transfer after fork | everything, or ids first | Everything first, ids later |
| History for new members | full, or since joining | Full (simplest); decide if privacy matters |
| Head broadcast interval | 15 to 60 s | About 30 s with debounce |
| Message timestamp trust | accept, or clamp | Clamp future values |
