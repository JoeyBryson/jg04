# Chat Invite System Audit: Follow-up

## Review basis

This follow-up reviews the current working tree at commit `0491f42` (`broken state, but backing up incase I break it further`) plus its uncommitted changes. The network implementation has moved since the original audit:

- `NwCore` now owns a `ChatSessionManager`.
- `ChatSessionManager` creates `ChatSession` instances.
- `ChatSession::spawn()` calls `gossip.subscribe()` and starts a receive loop.
- The old `chat_connector.rs` implementation has been replaced by `network/groupchat/session.rs`.

`cargo check --manifest-path rust-core/Cargo.toml` passes. It emits warnings, including unused imports and dead-code warnings, but no compilation errors.

## Executive summary

The sweeping refactor addresses the most visible form of findings 1 and 2: both locally created chats and accepted invites are now submitted to a shared session manager, and that manager attempts to create a gossip subscription and receive loop. Startup also restores stored chats through the same path.

The refactor does not yet make database state and network state transactional or durable. A local chat is still inserted into SQLite before invitation success, invite failures are logged from a detached task, and the acceptance response is sent after queueing session creation rather than after the session has successfully subscribed. The database still has no chat or invite lifecycle state.

### Status at a glance

| Finding | Status | Summary |
| --- | --- | --- |
| 1. Local chat has no connector | Addressed, with residual failure handling | Local creation calls `ChatSessionManager::add_chat`; session creation subscribes to gossip. Failures are logged and the caller is not informed. |
| 2. Accepted invite has no connector | Addressed, with an ack race | Acceptance calls `add_chat`, but `ChatInviteAccepted` is returned before the asynchronous subscription is confirmed. |
| 3. Fire-and-forget invite persistence | Open | The chat is persisted and session creation is queued before invite completion; failures leave the chat and session state unresolved. |
| 4. Contact bootstrap is circular | Open | Acceptance still requires `get_nw_contact(sender_id)` before processing the invite. |
| 5. Malformed or duplicate payloads | Open | Database constraints and a transaction help contain failures, but there is no protocol-level validation or idempotent duplicate handling. |
| 6. Chats with no members | Partially addressed | Reads reject empty chats and writes are atomic, but writes still permit empty member lists. |
| 7. Membership versus gossip swarm | Open | Every stored member is still passed as a gossip bootstrap ID, including pending and possibly local members. |
| 8. Enrollment is not fully confirmed | Open | The acknowledgment confirms only the topic ID, and is sent before the accepted peer has confirmed subscription success. |
| 9. Concurrent/repeated invites | Open | There is no invite key, deduplication, lifecycle state, or synchronization around repeated creation/invitation. |
| 10. Crash/restart recovery | Partially addressed | Startup recreates sessions for stored chats, but it cannot distinguish active, pending, failed, or previously interrupted invitations. |

## Finding 1: Local chats are created in the database but never subscribed to gossip

**Status: Addressed in the normal path; residual reliability gap remains.**

`NwCore::crate_chat()` now:

1. Builds the chat.
2. Starts the invitation task.
3. Persists the chat.
4. Calls `self.chat_session_manager.add_chat(chat)`.

`ChatSessionManager` receives the command and calls `ChatSession::spawn()`. `ChatSession::spawn()` calls `gossip.subscribe(topic_id, bootstrap_ids)`, splits the connection, and starts the receive loop. This removes the original missing-runtime-registration bug and means an immediate send is ordered after the add command in the manager's channel.

The fix is not complete from an error-handling perspective:

- `add_chat()` only queues a command; it does not mean that `gossip.subscribe()` succeeded.
- `ChatSessionManager` logs subscription failures and drops the failed session.
- `send_message()` can be accepted into the manager queue even when no session was created; the manager logs `no active chat session` and does not return that failure to the caller.
- The session manager replaces an existing session in its `HashMap` without explicitly handling duplicate registration.

The original claim that no runtime connector is created is no longer accurate. The remaining issue is that registration is asynchronous and failure is not observable through the API.

## Finding 2: Received invites are acknowledged but do not create a receive/send subscription

**Status: Addressed at the registration level; the acknowledgment still races activation.**

After rewriting the invited member, `ControlProtocol::accept()` now:

- inserts the chat into the database;
- calls `self.chat_session_manager.add_chat(chat)`; and
- sends `ChatInviteAccepted`.

The queued add command eventually creates a `ChatSession`, subscribes to gossip, and starts a receive loop. Therefore the original missing accepted-chat subscription has been addressed in code.

However, `add_chat()` returns after `try_send()` succeeds, not after `ChatSession::spawn()` succeeds. The peer can receive `ChatInviteAccepted` while the local gossip subscription is still pending or has already failed. This makes the response an acceptance of the database mutation and queue operation, not proof that the chat is live.

There is also no rollback if session creation fails after the database insert.

## Finding 3: Invite creation is fire-and-forget, so failures leave stale chats behind

**Status: Open and still high risk.**

`NwCore::crate_chat()` persists the chat synchronously, submits it to the session manager, and separately spawns `control_protocol::invite_chat_members(...)`. The caller returns before any remote invitation succeeds.

The invitation routine retries for up to approximately one minute per pending member. On final failure it returns an error, but the detached task only logs that error. There is no database state transition to `failed`, no deletion or rollback of the chat, and no user-visible result. The chat remains stored with pending members and an attempted session.

The new `mark_nw_chat_member_joined()` update is useful after a successful acknowledgment, but it does not solve failed invitations or make the overall chat active only after confirmation.

## Finding 4: The contact bootstrap path is circular

**Status: Open.**

`ControlProtocol::accept()` still begins by calling `db_client.get_nw_contact(sender_id)`. If the sender is not already in `contacts`, the invite is rejected before the payload is processed.

The refactor changes session registration but does not change this trust/bootstrap dependency. A first-time peer still needs an existing contact record or a separate identity/trust enrollment path.

## Finding 5: Duplicate or malicious chat payloads are not guarded

**Status: Open, with better atomic failure containment.**

`add_nw_chat()` now inserts the chat and all members in a SQLite transaction. That prevents a failed member insert from leaving a partially inserted chat. The schema also continues to enforce unique topic IDs and `(topic_id, endpoint_id)` pairs, valid status values, and foreign keys to contacts.

Those constraints are not protocol validation. The receiver still accepts a full peer-supplied `NwChat` without checking:

- that the topic is new or is an idempotent retry;
- that members are unique and non-empty;
- that the local identity appears correctly;
- that the sender is one of the declared members;
- that all member identities are trusted or known; or
- that names and membership data are consistent with the connection identity.

A duplicate topic or member currently fails at the database layer and becomes an accept error rather than a well-defined duplicate-invite response.

## Finding 6: The app can silently create chats with no active members

**Status: Partially addressed.**

The reader still rejects chats with no members, while `add_nw_chat()` now uses a transaction so the chat row is not left behind if a member insertion fails. That narrows the partial-write window.

The writer still accepts an empty `members` vector, and the schema does not enforce a minimum member count. An invalid empty chat can therefore still be inserted and will fail later when read. This remains a validation gap rather than a fully fixed lifecycle.

## Finding 7: Membership is not reconciled with the actual gossip swarm

**Status: Open.**

`ChatSession::spawn()` builds bootstrap IDs from every `chat.members` entry:

```rust
let bootstrap_ids = members
    .iter()
    .map(|member| member.contact.endpoint_id)
    .collect();
```

It does not filter pending members, exclude the local endpoint, remove duplicates, or distinguish an invite hint from an active/reachable peer. The session manager makes subscription creation more consistent, but the bootstrap model itself is unchanged in substance.

## Finding 8: The protocol never confirms chat enrollment beyond the single invite round trip

**Status: Open, and now more explicit because of the session manager queue.**

`ChatInviteAccepted { topic_id }` confirms only that the receiver parsed the invite, inserted the database row, and queued `add_chat()`. It does not confirm that:

- `ChatSession::spawn()` completed;
- `gossip.subscribe()` succeeded;
- the receive loop is running;
- the membership set was validated; or
- the peer is reachable on the chat topic.

The protocol still needs a readiness acknowledgment, or `add_chat()` needs an awaitable result that is completed only after subscription succeeds.

## Finding 9: Race conditions around concurrent invites and repeated chat creation

**Status: Open.**

The database still tracks only chats, members, and a binary pending/joined member status. There is no invite ID, attempt ID, lifecycle status, uniqueness rule for an invite operation, or explicit handling for a repeated `ChatInvite`.

The manager stores sessions by topic and replaces an existing session when another `Add` command uses the same topic. That avoids multiple entries in the map, but it is not a protocol-level idempotency policy and does not prevent repeated database inserts or repeated remote invitations.

## Finding 10: The invite flow has no durable recovery model after process crash or app restart

**Status: Partially addressed.**

`NwCore::spawn()` loads stored chats and calls `chat_session_manager.add_chat(chat)` for each one. This improves restart behavior for chats that are already present in the database: their gossip sessions are recreated automatically.

The database still cannot say whether a chat was active, pending, failed, or interrupted during an invite. On restart, every stored chat is treated as session-worthy, including chats whose invitations previously failed. Pending invite retries are not persisted or resumed, and session creation failures are only logged. Runtime rehydration is improved, but durable recovery is not solved.

## Updated root cause

The original missing connector path has been repaired through `ChatSessionManager`, but the two notions of truth are still not joined by a durable state machine:

- SQLite records a chat before remote membership is confirmed.
- The session manager attempts runtime activation asynchronously.
- The invite protocol acknowledges before runtime activation is confirmed.
- Failures are logged rather than represented in persistent state.

The current design is therefore closer to "database insert plus best-effort session startup" than to an activation protocol.

## Recommended next steps

1. Add explicit chat/member lifecycle states such as `pending`, `active`, and `failed`.
2. Make session registration return an awaitable activation result, and send `ChatInviteAccepted` only after subscription succeeds.
3. Persist invite attempts and failure state; make retries resume safely after restart.
4. Add protocol validation for sender identity, local membership, duplicate topics, duplicate members, and empty member sets.
5. Define idempotent behavior for repeated invites and duplicate session registration.
6. Filter gossip bootstrap IDs to valid remote peers and treat membership as an advisory bootstrap hint.
7. Add tests for immediate send after creation, subscription failure after acceptance, failed invite cleanup/state, duplicate invites, unknown senders, and restart recovery.

## Conclusion

The sweeping changes successfully address the original "chat exists but no connector is registered" defect for both local creation and inbound acceptance in the normal successful path. They do not yet establish that the connector is live before reporting success, and they do not prevent or recover from stale persisted chats. Findings 3, 4, 5, 7, 8, and 9 remain open; findings 6 and 10 are only partially addressed.
