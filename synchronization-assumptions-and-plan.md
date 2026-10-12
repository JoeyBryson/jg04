# Synchronization: Assumptions to Test and Implementation Plan

Follow-up to [synchronization-protocol-report.md](synchronization-protocol-report.md).

## 0. Done so far

The `delivered_from` bug is fixed in [signed.rs](rust-core/src/network/messaging/signed.rs). `SignedMessage` now carries an `author: EndpointId`, and `verify_and_decode` verifies the signature against that author instead of the relaying neighbor. Two unit tests cover it: a normal round trip, and a swapped author being rejected. This changes the gossip wire format, which is fine pre-release.

Not yet fixed (planned below): the signature does not bind `topic_id`, and nothing stores signatures or deduplicates messages.

---

## A. Current assumptions

Status key: **Read/Confirmed** = supported by code or authoritative documentation, **Verified** = covered by a passing test, **Accepted assumption** = intentionally proceeding without a test for now, **Unverified/Risk** = still needs evidence or a decision.

### A1. Gossip behavior

| # | Assumption | Status | How to test | My thoughts |
|---|---|---|---|---|
| G1 | `NeighborUp` fires on both sides when two peers connect, including reconnects, and for a peer that was offline and returns. | Verified by integration test | [gossip_neighbor_events.rs](rust-core/tests/gossip_neighbor_events.rs) checks both peers on initial connection and after Bob restarts with the same identity. | Confirmed; continue to use `NeighborUp` as a sync trigger. |
| G2 | Gossip delivers messages to different peers in different orders and may drop some. | Accepted conservative assumption | No test planned. Design sync and persistence to tolerate reordering and loss. | Assume this can happen. |
| G3 | Maximum gossip message size is 4096 bytes by default (`DEFAULT_MAX_MESSAGE_SIZE`). | Read; accepted for now | The dependency default is 4096 bytes. No need to test the boundary before designing sync. Account for envelope overhead when choosing a content limit. | Assume true for now. |
| G4 | Messages relayed through a third peer now verify. | Verified by integration test | [gossip_relay_messages.rs](rust-core/tests/gossip_relay_messages.rs) forms a swarm, selects a non-neighbor source/target pair, and confirms the target receives the message from a relay and verifies the signature using the embedded author. The decoder's author selection is also covered by the `signed.rs` unit test. | Tested; no further topology-specific work planned. |
| G5 | Broadcasting while having no neighbors returns `Ok` and the message is only stored locally. | Accepted assumption | No test planned now. Keep local persistence independent of current neighbor availability; sync must later carry locally stored messages. | Assume true. |
| G6 | iroh-gossip may re-deliver or deduplicate identical payloads. | Accepted conservative assumption | No test planned. Require application/database deduplication regardless of gossip's behavior. | Assume duplicate delivery is possible. |

### A2. Database

| # | Assumption | Status | How to test | My thoughts |
|---|---|---|---|---|
| D1 | One writer thread serializes all writes, so chain recomputation cannot race. | Read: `spawn_workers` creates a single `DbWriter` thread | Covered by design; also test concurrent `add_nw_message` calls from two tasks. |  |
| D2 | Foreign keys are enabled on every database connection. | Fixed and verified | `DbManager::start_conn` enables the pragma per connection. The regression test checks it on read-only and read-write connections and confirms deleting a chat cascades to messages. | Fixed; [database_connections_enable_foreign_key_enforcement](rust-core/src/database/client_test.rs) passes. |
| D3 | `messages` has no uniqueness today, so replaying the same message creates a duplicate row. | Read | Test: add the same `NwMessage` twice, count rows. |  |
| D4 | `ui_reads` orders by `sent_at, id` and `nw_reads` orders by `id`. | Read | Becomes a test once the canonical order is introduced. |  |
| D5 | SQLite row-value comparison `(sent_at, endpoint_id, message_id) > (?, ?, ?)` compares BLOB columns bytewise and consistently across devices. | Confirmed by SQLite documentation | Row-value comparisons compare components left-to-right; two BLOB values compare with `memcmp()`. This is deterministic when `sent_at` is stored as INTEGER and both IDs are consistently bound/stored as BLOBs (as planned). Avoid mixed TEXT/BLOB representations. A small query test is still useful as an implementation regression check, but this is no longer an open design assumption. |  |
| D6 | Rewriting the tail of a chat (UPDATE of `seq` and `chain_hash` for every row after the insert point) is cheap enough on the writer thread. | Unverified | Benchmark: 10k, 100k messages, insert at position 0 in one transaction, measure time. This is the decisive number for choosing Option A. The existing `performance_smoke_analysis` in [tests.rs](rust-core/src/network/tests.rs) is a model for this. | A message-age limit could bound how far back a new insertion can sort, but rejecting older messages would also reject legitimate history from peers that were offline longer than the limit. A time window also does not bound message count in a busy chat. Keep this as a benchmark; don't impose a sync age limit unless losing older history is an accepted product rule. |
| D7 | The schema can change without migrations. | Read: comment in [mod.rs](rust-core/src/database/mod.rs) says reset the db | No test needed. Revisit before real users. |  |
| D8 | `sent_at` is milliseconds since the Unix epoch, produced in `NwService::send_message`. | Read | None. |  |

### A3. Networking and protocol

| # | Assumption | Status | How to test | My thoughts |
|---|---|---|---|---|
| N1 | `ProtocolHandler::accept` gives an authenticated remote identity via `connection.remote_id()`. | Read: the method exists in iroh 1.0.3 | Integration test: handler logs `remote_id()`, compare with the initiator's endpoint id. |  |
| N2 | `Router::builder(endpoint).accept(ALPN, handler)` is sufficient to serve a third ALPN alongside control and gossip. | Accepted assumption | User expects this to work based on the existing iroh Router pattern. Revisit only if the protocol spike fails. | Assume true; no pre-implementation spike needed. |
| N3 | A multi-round exchange on one bi stream with length-prefixed frames works with iroh's `SendStream`/`RecvStream`. | Accepted assumption | User expects standard bidirectional iroh streams to support the exchange. Validate framing in protocol tests during implementation. | Assume true; no separate 5 MB spike needed. |
| N4 | An initiator can create a sync connection from only an `Endpoint` (no `Router`). | Read: `ChatInvitesHandle` uses `router.endpoint().connect(...)`, which is available on `Endpoint` directly | Covered by the N2 spike. |  |
| N5 | Two peers opening syncs to each other at the same time is a realistic race. | Unverified | Test: both sides get `NeighborUp` at about the same time (G1). Decide the tie-break (lower endpoint id initiates). |  |
| N6 | The invite flow gives a joined invitee a usable bootstrap, so they will get `NeighborUp` with existing members and can sync history. | Accepted assumption | `ChatSession::spawn` bootstraps from `Joined` members except self; G1 verifies gossip emits neighbor events on join/reconnect. | Assume true; exercise this in the sync integration tests rather than as a separate spike. |
| N7 | `ControlProtocol` does not check membership. Any peer that knows our endpoint id can send an invite. | Read | Out of scope here, but the sync responder must not copy this. See [chat-invite-system-audit.md](chat-invite-system-audit.md). |  |

### A4. Design assumptions

| # | Assumption | Status | Notes | My thoughts |
|---|---|---|---|---|
| X1 | Equal chain heads imply identical ordered message sets. | True by construction, if the hash input is the `message_id` and the order is canonical. | Permutation test (see plan). |  |
| X2 | Out-of-order insert is common enough that the rehash path is hot. | Unverified | See G2 and D6. |  |
| X3 | Group sizes are small (up to dozens). | Assumed | If chats can reach hundreds of members, head gossip cost changes. Confirm with product intent. |  |
| X4 | Chats are never huge (up to tens of thousands of messages). | Assumed | Drives D6. |  |
| X5 | The chat has one shared ledger ordered canonically across all authors. | Selected design | Every peer uses `(sent_at, endpoint_id, message_id)`; deterministic message IDs, not local row IDs, break ties. | Option A selected. |
| X6 | Newly invited members should receive full history. | Open decision | Not a technical assumption; needs a product answer. |  |
| X7 | Every message author is present in the local `contacts` table. | Confirmed by product model | Sync must reject unknown authors; add/enforce `messages.endpoint_id` foreign key to `contacts(endpoint_id)`. This relies on D2's per-connection foreign-key check being fixed if needed. |  |

---

## B. Implementation plan

### Recommendation

Use **Option A**, one global ledger ordered canonically by `(sent_at, endpoint_id, message_id)`. Tail appends are cheap; an older arrival requires rehashing the affected suffix. At the expected few-hundred-message group-chat scale, this is expected to be acceptable. Benchmark near the proposed history cap before using the same policy for a large public forum, and batch sync inserts so the suffix is rehashed only once.

Keep chain identity independent of local row IDs and hash signed `message_id`, not raw content, so future delete/tombstone events can be represented as new ledger entries.

### Phase 1: Benchmark the selected chain

G1, G4, D2, and D5 are verified. G2, G3, G5, G6, N2, N3, and N6 are accepted assumptions for now. Option A is selected; D6 is a scale/latency check, not a chain-selection gate. D3 can be covered as part of database deduplication implementation.

Benchmark full-suffix rehashes using the intended schema and batch-write path at a few hundred rows, 10k, 100k, and near the proposed maximum. Measure transaction latency, writer queue delay, WAL growth, and read latency with a reader open. Large sizes are public-forum guardrails, not expected group-chat workloads.

Working policy: there is one canonical ledger per chat; any joined peer can reconcile the shared ledger and provide stored messages. Verify each message against its original author's signature, regardless of which peer serves it.

### Phase 2: Signed message format

Files: [signed.rs](rust-core/src/network/messaging/signed.rs), [session.rs](rust-core/src/network/chat/session.rs), [persistence/sessions.rs](rust-core/src/network/persistence/sessions.rs).

1. Signed payload becomes `{ topic_id, sent_at, kind }`, binding the chat and content to the signature; the envelope also carries the author and signature.
2. Add deterministic `message_id` from canonical signed bytes and retain the payload/signature for verification and serving.
3. Enforce unique `(topic_id, message_id)` and persist the signed message before broadcast. Use the direct new gossip message shape; all existing data will be reset, so no compatibility envelope, format version, or migration is needed.
4. On receive, verify signature, topic, and current membership, then insert idempotently. The local row id is not part of the canonical order.
5. Enforce the content limit from G3.

Tests: round trip, wrong-topic rejection, tampered payload rejection, deterministic ID, duplicate no-op, and local persistence before broadcast.

### Phase 3: Database and chain

Files: [schema.sql](rust-core/src/database/sql/schema.sql), [writes.rs](rust-core/src/database/workers/writes.rs), [nw_reads.rs](rust-core/src/database/workers/nw_reads.rs), [ui_reads.rs](rust-core/src/database/workers/ui_reads.rs), [macros.rs](rust-core/src/database/macros.rs), [manager.rs](rust-core/src/database/manager.rs).

1. Schema: add `message_id`, `signature`, `seq`, and `chain_hash`; unique `(topic_id, message_id)`; index canonical order `(topic_id, sent_at, endpoint_id, message_id)`; retain foreign keys to chats and contacts.
2. In one writer transaction, check Joined membership and deduplicate, insert one message or a verified batch, find the earliest changed canonical position, and recompute `seq` and `chain_hash` to the tail.
3. Sync batches must be inserted before one suffix rehash, not rehashed once per message. Commit the received batch and chain updates atomically.
4. Read APIs return chat head/count, chain samples, and messages by canonical sequence/key. UI reads use the same canonical order.
5. Batch writes emit one `ChatDataChanged` and one `ChatHeadersChanged`.
6. `PRAGMA foreign_keys = ON` is now set by `DbManager::start_conn` for reader and writer connections; the regression test confirms cascade enforcement.

Tests: permutation test (same message set inserted in different orders produces the same head), duplicate no-op, head/middle/tail insertion against a full recomputation, batch rehashes once, timestamp ties resolve by author then message ID, and non-member/wrong-topic messages are rejected. Benchmark representative suffix sizes and writer queue delay.

### Phase 4: Sync protocol (manual trigger first)

New module `rust-core/src/network/sync/`: `messages.rs`, `framing.rs`, `handler.rs` (responder), `initiator.rs`. Wiring in [network/mod.rs](rust-core/src/network/mod.rs) and [service.rs](rust-core/src/network/application/service.rs).

1. `SYNC_ALPN = b"jg04/sync/1"`. Register it on the `Router`.
2. Frame format: 4-byte length prefix then a postcard-encoded enum. Max frame size 256 KiB; reject larger.
3. Responder:
   - take `connection.remote_id()`,
   - read `Hello { topic }`,
   - reject if the topic is unknown or the remote is not a `Joined` member,
   - run the exchange.
4. Exchange (v1):
   1. Exchange `Hello { topic, my_head, my_count }`; equal heads finish immediately.
   2. Exchange exponentially spaced `(seq, chain_hash)` samples and locate the latest common prefix.
   3. Exchange messages after that prefix in bounded chunks. A peer may serve any messages it has stored, regardless of their original author.
   4. Verify each embedded author's signature, topic, and current membership; deduplicate and insert each received batch, then rehash the ledger suffix once.
   5. Exchange final heads after commit. If new messages arrived during sync or heads differ, run another bounded round.
5. Per-frame timeout and total session timeout (longer than the invite 5 s).
6. Initiator actor, modeled on `ChatInvitesHandle`: command `SyncWith { topic, peer }`, at most one in-flight sync per `(topic, peer)`, per-peer rate limit.
7. Take an `Endpoint` clone so the initiator can be created before `ChatSessionsHandle`, avoiding the router ordering problem.

Tests (integration, using the `tests/common` harness):

- Two peers with different canonical ledgers exchange missing rows and converge to identical heads.
- A peer can serve a stored message authored by another member; the receiver verifies the original author's signature.
- One node offline during a burst of messages catches up.
- A newly joined member receives history (N6).
- Non-member connection rejected.
- Forged author, wrong topic, oversized frame rejected.
- Connection dropped mid-sync leaves no partial state and a retry succeeds.
- Both sides initiating at once ends in a single consistent result.

### Phase 5: Automatic triggers

1. `receive_loop` handles `NeighborUp(peer)` by requesting global-ledger reconciliation with that joined peer.
2. Signed `Head` gossip with debounce may detect drift between peers already connected; it supplements reconnect sync.
3. `ChatSession` receives a handle to the sync initiator from `ChatSessionsHandle::spawn`.

Tests: automatic catch-up after reconnect without any manual trigger; no sync storm when many members announce at once.

### Phase 6: Hardening and optimizations (only if needed)

- Id exchange before message transfer.
- Checkpointed chain hashes if D6 shows rehash cost is a problem.
- Clamp `sent_at` too far in the future.
- UI: handle messages inserted mid-history; optional "syncing" indicator.
- Delete messages (separate project; design choice already made by hashing `message_id`).

### Suggested order and gates

| Gate | After | Question | My thoughts |
|---|---|---|---|
| 1 | Phase 1 | Does batched rehash meet the write-latency target at normal and maximum intended history sizes? |  |
| 2 | Phase 3 | Does the permutation test prove identical heads for identical message sets? |  |
| 3 | Phase 4 | Do two peers converge reliably with a manual sync trigger? |  |
| 4 | Phase 5 | Does reconnect/head-triggered sync work without creating sync storms? |  |

### Open product decisions

| Question | Suggestion | My thoughts |
|---|---|---|
| Should new members see full history? | Yes for now (X6). Needs a product answer. |  |
| Maximum message content size? | About 3 KB (G3). |  |
| Allowed `sent_at` skew into the future? | A few minutes. |  |
| Expected maximum group size and chat length? | Needed to confirm X3 and X4. |  |

## Overall thoughts

Add any broader feedback, concerns, or decisions here:

<textarea rows="5" style="width: 100%;" placeholder="Write your thoughts here..."></textarea>
