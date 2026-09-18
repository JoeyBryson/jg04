# Chat Invite System Audit

## Scope
This review targets the invite flow and state lifecycle in:
- [rust-core/src/network/core.rs](rust-core/src/network/core.rs)
- [rust-core/src/network/control_protocol.rs](rust-core/src/network/control_protocol.rs)
- [rust-core/src/network/chat_connector.rs](rust-core/src/network/chat_connector.rs)
- [rust-core/src/database/sql/schema.sql](rust-core/src/database/sql/schema.sql)

## Executive summary
The invite design is split between a database-backed chat record and a separate runtime gossip subscription. The critical problem is that these two layers are not kept in sync.

Creating or accepting a chat writes a row to SQLite, but it does not reliably create the `NwChatConnector` that actually subscribes to gossip and sends/receives messages. The result is a state where the app thinks a chat exists, yet the network layer is missing or stale. Several edge cases also make invites fail silently or leave the app in a permanently inconsistent state.

---

## Finding 1: Local chats are created in the database but never subscribed to gossip
Severity: Critical

In [rust-core/src/network/core.rs](rust-core/src/network/core.rs), `crate_chat()`:
1. builds a `NwChat`
2. spawns an async invite process
3. writes the chat to the database
4. returns

It never creates a `NwChatConnector` for the new topic, even though the app later looks up connectors from `self.chat_connectors` in `send_message()`.

Relevant logic:
- `self.chat_connectors` is initialized only during `NwCore::spawn()` for chats loaded from the database.
- `crate_chat()` does not insert a new entry into `self.chat_connectors`.
- `send_message()` does `self.chat_connectors.get(&topic_id)` and returns an error if missing.

This means a freshly created chat can exist in SQLite and still fail at runtime with:
- `No active connector for topic ID: ...`
- no live gossip subscription
- no local message delivery path
- no receive loop for the chat

This is a fundamental state bug, not just a missing optimization.

### Why it is especially bad
The app treats the database as canonical state, but the network runtime is actually the real operating state. Creating a chat in one layer without the other makes the app internally inconsistent.

### Edge cases
- User creates a chat, then immediately sends a message; it fails.
- App restarts; the reconnect path rebuilds connectors for stored chats, but the just-created chat is only restored if it was already persisted before the app died.
- The invite succeeds but the local connector never starts, so the user can see the chat but cannot use it.

---

## Finding 2: Received invites are acknowledged but do not create a receive/send subscription
Severity: Critical

In [rust-core/src/network/control_protocol.rs](rust-core/src/network/control_protocol.rs), `accept()` handles an inbound `ControlMessage::ChatInvite` by:
- fetching the sender from the database
- rewriting members to swap local `self.profile.contact` with the sender
- inserting the chat into the database
- sending `ChatInviteAccepted`

It does not create a `NwChatConnector` for the accepted chat.

Equivalent issue to Finding 1, but on the inbound side:
- the accepted chat exists in the database
- the app may display it
- but there is no `gossip.subscribe(topic_id, ...)` pipeline for that chat
- the app cannot later send or receive messages on the topic unless some other startup path recreates it

This means the invite handshake is only a database mutation, not a network session creation.

### Consequence
The system behaves as if “invite accepted” means “chat is live,” when really it only means “database stored a row and we replied to the inviter.”

---

## Finding 3: Invite creation is fire-and-forget, so failures leave stale chats behind
Severity: High

`crate_chat()` does this:
- builds the chat
- launches an async background task to call `invite_chat_members()`
- writes the chat row to the database immediately
- returns without waiting for the invitation to succeed

The background invite task can fail for many reasons: timeout, unreachable peer, connection refusal, peer rejects the invite, peer has no matching database state, or a bad ALPN path. Yet the local database has already accepted the chat as valid.

This produces orphaned chats:
- there is a row in `chats`
- there may be rows in `chat_members`
- but no actual network membership was established
- no connector is created
- the user can still click into the chat and see an empty or broken state

### Why this is poor protocol design
The system treats a database insert as success before network confirmation. That is exactly backwards for a network handshake.

### Better behavior
- create invite state with a pending or failed status
- wait for a remote ack before persisting as active
- roll back or mark as failed when invitation times out
- only create connector once the invite is confirmed

---

## Finding 4: The contact bootstrap path is circular and can reject valid invites
Severity: High

In [rust-core/src/network/control_protocol.rs](rust-core/src/network/control_protocol.rs), `accept()` calls:

`self.db_client.get_nw_contact(sender_id)`

This will fail if the sender is not already known in the contacts table.

But the invite system is exactly how a user discovers a remote contact and establishes a chat relationship. That means the onboarding flow is circular:
1. you need a contact record to accept an invite
2. the invite is the thing that should establish that contact
3. but the accept path refuses the invite unless the contact already exists

This creates a dead end for first-time or not-yet-synced contacts. The app assumes import/known-contact state before network trust is established, which is incompatible with a peer-to-peer invite system.

### Impact
- A fresh peer cannot be invited into a chat unless they were already known.
- A peer who’s never been stored locally cannot join a chat through this protocol.
- The network handshake implicitly requires out-of-band identity registration.

---

## Finding 5: Duplicate or malicious chat payloads are not guarded
Severity: High

The `ControlMessage::ChatInvite` carries a full `NwChat` payload from the peer. The receiving side does a blind `add_nw_chat(chat)` in [rust-core/src/network/control_protocol.rs](rust-core/src/network/control_protocol.rs).

There is no validation for:
- duplicate topic IDs
- repeated member entries
- self-membership misrepresentation
- invalid or untrusted contact names
- local membership that does not match the sender's actual identity
- chat payloads that include endpoints the receiver does not know or trust

The database schema in [rust-core/src/database/sql/schema.sql](rust-core/src/database/sql/schema.sql) adds some constraints, but not enough:
- `chat_members` has `PRIMARY KEY (topic_id, endpoint_id)`
- `chats.topic_id` is unique
- `chat_members.endpoint_id` references `contacts.endpoint_id`

These constraints can cause failed inserts, but they do not prevent malicious or malformed invite data from getting as far as the database layer.

### Example edge case
If a remote user reuses a topic ID that already exists locally, the insert can fail with a DB uniqueness error. The accept path treats that as an error but the rest of the system does not cleanly recover or present a meaningful error state.

---

## Finding 6: The app can silently create chats with no active members
Severity: Medium

The database reader in [rust-core/src/database/workers/nw_reads.rs](rust-core/src/database/workers/nw_reads.rs) rejects a chat if it has no members:

`if chat.members.is_empty() ... return Err(...)`

But the writer in [rust-core/src/database/workers/writes.rs](rust-core/src/database/workers/writes.rs) does not enforce that a chat has at least one valid member before insertion. This leaves a window where malformed or partial invite data produces a row that the reader later rejects.

The invite system can therefore create a half-valid chat record and then the app fails later during reads, without a clear recovery path.

---

## Finding 7: Membership is not reconciled with the actual gossip swarm
Severity: Medium

`NwChatConnector::spawn()` in [rust-core/src/network/chat_connector.rs](rust-core/src/network/chat_connector.rs) subscribes with a bootstrap list built from `chat.members`:

`let bootstrap_ids = members.iter().map(|member| member.endpoint_id).collect();`

This is a fragile assumption. It is not clear that every member is online or reachable; it also does not exclude the local user. If the local user appears in the chat’s member list, the bootstrap list can include the local endpoint ID, which is not useful for a peer-to-peer network of distinct devices.

A few edge cases:
- all members are offline when the chat is created
- one member is never reachable, so the swarm never forms
- a stale membership list includes users no longer in the chat
- the chat topic is valid but the bootstrap path never converges because the set is empty or invalid

This system should treat member list as an invite hint, not a reliable swarm bootstrap mechanism.

---

## Finding 8: The protocol never confirms chat enrollment beyond the single invite round trip
Severity: Medium

The `send_chat_invite()` handshake in [rust-core/src/network/core.rs](rust-core/src/network/core.rs) sends a `ChatInvite`, waits for a single `ChatInviteAccepted`, and then exits. That is only a half-acknowledgment.

It does not confirm that:
- the peer actually created a local connector
- the peer successfully inserted the chat into its own database
- the peer subscribed to the topic
- the peer’s chat membership list matches the inviter’s expected members
- the peer is reachable for subsequent chat traffic

The protocol stops at “I got a response bytes object” rather than “the peer is ready to participate in the chat.”

This is not enough for a reliable distributed chat protocol.

---

## Finding 9: Race conditions around concurrent invites and repeated chat creation
Severity: Medium

The app can issue multiple invites for the same topic or same member set without deduplication. The same chat topic can also be generated twice by random collisions, although the risk is low.

More importantly:
- the chat is created immediately, before confirmation
- background retries are run per member
- the same chat may be re-invited if the user retries or reopens the flow
- the database does not track invite state, so there is no way to tell whether a chat is pending, active, or failed

This makes retries idempotent state management impossible.

---

## Finding 10: The invite flow has no durable recovery model after process crash or app restart
Severity: Medium

The system has no persisted state for:
- pending invites
- invite retries
- membership confirmation
- chat lifecycle status
- failed invites or timeouts

After a crash, the database may contain chats that are not actually live, while the runtime has no record of which invites were in flight. On restart, `NwCore::spawn()` rebuilds connectors for all chats in the database, but it does not repair chats that were partially created or never fully bootstrapped.

This is a classic “disappearing state” bug: the database is not enough to reconstruct a valid chat session.

---

## High-confidence root cause
The underlying design issue is a mismatch between two different notions of truth:

- the database says “chat exists”
- the runtime says “chat is connected and active”

The app writes the first one immediately and the second one only during startup, but never on invite creation or acceptance. As a result, the invite handshake is not an activation protocol; it is just a database mutation and a best-effort network call.

---

## Recommended redesign
1. Split chat state into explicit phases: `pending`, `active`, `failed`.
2. Do not insert a chat as active before the invite is acknowledged.
3. Create and register a `NwChatConnector` as part of invite acceptance, not only during app startup.
4. Add idempotent invite keys and topic dedup validation.
5. Require a contact record or explicit trust step before accepting an invite from a new endpoint.
6. Have the peer ack a richer protocol than just `ChatInviteAccepted` — it should confirm the topic and membership are ready.
7. Treat the member list as advisory, not as guaranteed swarm bootstrap data.
8. Add integration tests covering:
   - invite accepted but connector missing
   - invite fails and chat remains pending
   - duplicate invite for same topic
   - new contact invite without existing contact record
   - chat creation followed by immediate send

---

## Conclusion
This invite system is not robust enough to be treated as a production chat protocol. The biggest issues are not small bugs; they are structural mismatches between database state, network state, and handshake semantics. The current design can leave the app with chats that appear valid but are impossible to use, and it can reject valid invite flows due to circular dependency on preexisting contact records.
