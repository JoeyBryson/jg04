//! test file for database writes
//! All tests passing proves:
//! 1) `set_nw_profile` is one-time setup. Any second call fails because profile setup is already
//!    complete, even if payload is identical.
//! 2) `set_nw_profile` rejects conflicting profile data after setup (same as any later call:
//!    profile is already set).
//! 3) `set_nw_profile` stores a profile that can be read back exactly (secret key + contact).
//! 4) On successful setup, `set_nw_profile` keeps the profile endpoint available in `contacts`,
//!    so later chat member inserts that reference the local endpoint satisfy foreign-key rules.
//! 5) `add_nw_contact` is idempotent for identical input. Re-adding the same endpoint/name pair
//!    is a no-op, not a duplicate row.
//! 6) `add_nw_contact` rejects conflicts for the same endpoint with a different name.
//! 7) Missing contacts are correctly reported as errors when read before insertion.
//! 8) `add_nw_chat` is idempotent for exact duplicates (same topic/name/members). Retry does not
//!    create a second chat or change member data.
//! 9) `add_nw_chat` rejects conflicting payloads for an existing topic (for example different
//!    name or different members), preventing accidental mutation through an insert API.
//! 10) `mark_nw_chat_member_joined` is idempotent. Repeating the join mark keeps the row valid
//!     and joined.
//! 11) `mark_nw_chat_member_joined` fails when the target row does not exist.
//! 12) `add_ui_contact` is not idempotent. Repeating the same UI add call fails on duplicate key,
//!     which matches strict "UI action happened already" behavior.
//! 13) `add_ui_contact` rejects malformed endpoint strings.
//! 14) `add_chat_ui` creates a valid topic id string and inserts a usable chat.
//! 15) `add_chat_ui` + `add_nw_chat` together preserve conflict detection even when reusing the
//!     generated topic id.
//! 16) `reset_database` is repeatable (safe to call twice) and clears profile/chats/contacts as
//!     observed through read APIs.
//! 17) `update_contact_name` updates normal contacts and also updates the profile name when the
//!     updated endpoint is the local profile endpoint.
//! 18) `update_contact_name` fails for unknown endpoints, so callers can detect invalid updates.
//! 19) `update_contact_name` is idempotent when setting the same final name again.
//! 20) `update_chat_members` replaces the member list for an existing chat and persists new
//!     statuses.
//! 21) `update_chat_members` rejects invalid updates (empty member list / unknown chat).
//! 22) `update_chat_members` is atomic with transaction rollback: if one new member insert fails
//!     (like a foreign-key failure), previously stored members remain unchanged.
//! 23) The write-layer conflict policy is operation-specific and preserved: one-time profile setup
//!     and UI duplicate adds fail, idempotent network insert paths retry safely, and successful
//!     writes round-trip through read APIs.

use anyhow::Result;
use iroh_gossip::proto::TopicId;

use super::test_utils::*;
use crate::network::{NwChat, NwChatMemberStatus};

#[test]
fn profile_write_is_one_time_and_conflicts_are_rejected() -> Result<()> {
    let writer = new_writer();
    let profile = profile();

    writer.set_nw_profile(profile.clone())?;
    assert!(writer.set_nw_profile(profile.clone()).is_err());

    let mut conflicting = profile.clone();
    conflicting.contact.name = "Other".to_string();
    assert!(writer.set_nw_profile(conflicting).is_err());

    let reader = reader_from(writer.conn);
    let stored = reader.get_nw_profile()?;

    assert_eq!(stored.secret_key.to_bytes(), profile.secret_key.to_bytes());
    assert_eq!(stored.contact, profile.contact);

    let stored_contact = reader.get_nw_contact(&profile.contact.endpoint_id)?;
    assert_eq!(stored_contact, profile.contact);

    Ok(())
}

#[test]
fn contact_write_is_idempotent_and_missing_reads_fail() -> Result<()> {
    let writer = new_writer();
    let alice = contact(1, "Alice");

    assert!(
        reader_from(connection())
            .get_nw_contact(&alice.endpoint_id)
            .is_err()
    );

    writer.add_nw_contact(alice.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let mut conflicting = alice.clone();
    conflicting.name = "Different".to_string();
    assert!(writer.add_nw_contact(conflicting).is_err());

    let reader = reader_from(writer.conn);
    assert_eq!(reader.get_nw_contact(&alice.endpoint_id)?, alice);

    Ok(())
}

#[test]
fn chat_write_round_trips_and_is_idempotent() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    let bob = contact(2, "Bob");

    writer.add_nw_contact(alice.clone())?;
    writer.add_nw_contact(bob.clone())?;

    let chat = chat(
        7,
        vec![
            member(alice.clone(), NwChatMemberStatus::Pending),
            member(bob.clone(), NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;
    writer.add_nw_chat(chat.clone())?;

    let reader = reader_from(writer.conn);
    let stored_chat = reader.get_nw_chat(&chat.topic_id)?;

    assert_eq!(stored_chat.name, chat.name);
    assert_eq!(member_keys(&stored_chat), member_keys(&chat));

    let mut conflicting = chat.clone();
    conflicting.name = Some("Conflicting name".to_string());

    let mut writer = new_writer();
    writer.add_nw_contact(alice)?;
    writer.add_nw_contact(bob)?;
    writer.add_nw_chat(chat)?;

    assert!(writer.add_nw_chat(conflicting).is_err());

    Ok(())
}

#[test]
fn marking_members_joined_is_idempotent_and_missing_rows_fail() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");

    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        11,
        vec![member(alice.clone(), NwChatMemberStatus::Pending)],
    );

    writer.add_nw_chat(chat.clone())?;

    writer.mark_nw_chat_member_joined(chat.topic_id, alice.endpoint_id)?;
    writer.mark_nw_chat_member_joined(chat.topic_id, alice.endpoint_id)?;

    assert!(
        writer
            .mark_nw_chat_member_joined(TopicId::from_bytes([12; 32]), alice.endpoint_id)
            .is_err()
    );

    let reader = reader_from(writer.conn);

    assert_eq!(
        reader.get_nw_chat(&chat.topic_id)?.members[0].status,
        NwChatMemberStatus::Joined
    );

    Ok(())
}

#[test]
fn convenience_writes_and_chat_conflicts_are_handled() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;

    writer.add_ui_contact(crate::ui::UiContact {
        name: alice.name.clone(),
        endpoint_id: hex::encode(alice.endpoint_id.as_bytes()),
    })?;

    assert!(writer
        .add_ui_contact(crate::ui::UiContact {
            name: alice.name.clone(),
            endpoint_id: hex::encode(alice.endpoint_id.as_bytes()),
        })
        .is_err());

    let chat_id = writer.add_chat_ui(
        vec![crate::ui::UiContact {
            name: alice.name.clone(),
            endpoint_id: hex::encode(alice.endpoint_id.as_bytes()),
        }],
        Some("Convenience".to_string()),
    )?;

    assert_eq!(chat_id.len(), 64);

    assert!(
        writer
            .add_ui_contact(crate::ui::UiContact {
                name: "Alice 2".to_string(),
                endpoint_id: hex::encode(alice.endpoint_id.as_bytes()),
            })
            .is_err()
    );

    assert!(
        writer
            .add_ui_contact(crate::ui::UiContact {
                name: "Invalid".to_string(),
                endpoint_id: "invalid".to_string(),
            })
            .is_err()
    );

    let topic_id = TopicId::from_bytes(
        hex::decode(chat_id)
            .unwrap()
            .try_into()
            .expect("generated topic ID must be 32 bytes"),
    );

    let mut conflicting_chat = NwChat {
        name: Some("Other".to_string()),
        members: vec![
            member(profile.contact.clone(), NwChatMemberStatus::Joined),
            member(alice.clone(), NwChatMemberStatus::Pending),
        ],
        topic_id,
    };

    assert!(writer.add_nw_chat(conflicting_chat.clone()).is_err());

    conflicting_chat.members.clear();

    assert!(writer.add_nw_chat(conflicting_chat).is_err());

    Ok(())
}

#[test]
fn reset_database_is_repeatable() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(alice.clone())?;

    writer.add_nw_chat(chat(
        17,
        vec![
            member(profile.contact, NwChatMemberStatus::Joined),
            member(alice, NwChatMemberStatus::Joined),
        ],
    ))?;

    writer.reset_database()?;
    writer.reset_database()?;

    let reader = reader_from(writer.conn);

    assert!(reader.get_nw_profile().is_err());
    assert!(reader.get_ui_profile()?.is_none());
    assert!(reader.get_nw_chats()?.is_empty());
    assert!(reader.get_ui_contacts()?.is_empty());

    Ok(())
}

#[test]
fn update_contact_name_updates_contact_and_profile_name() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(profile.contact.clone())?;
    writer.add_nw_contact(alice.clone())?;

    writer.update_contact_name(alice.endpoint_id, "Alice Renamed".to_string())?;
    writer.update_contact_name(profile.contact.endpoint_id, "Me Renamed".to_string())?;

    assert!(writer
        .update_contact_name(contact(9, "Missing").endpoint_id, "Nope".to_string())
        .is_err());

    let reader = reader_from(writer.conn);

    let updated_alice = reader.get_nw_contact(&alice.endpoint_id)?;
    assert_eq!(updated_alice.name, "Alice Renamed");

    let updated_profile = reader.get_nw_profile()?;
    assert_eq!(updated_profile.contact.name, "Me Renamed");

    Ok(())
}

#[test]
fn update_chat_members_replaces_members_and_rejects_invalid_updates() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    let bob = contact(2, "Bob");
    let alice_endpoint = alice.endpoint_id;
    let bob_endpoint = bob.endpoint_id;

    writer.add_nw_contact(alice.clone())?;
    writer.add_nw_contact(bob.clone())?;

    let chat = chat(
        21,
        vec![member(alice.clone(), NwChatMemberStatus::Pending)],
    );

    writer.add_nw_chat(chat.clone())?;

    writer.update_chat_members(
        chat.topic_id,
        vec![
            member(alice.clone(), NwChatMemberStatus::Joined),
            member(bob.clone(), NwChatMemberStatus::Pending),
        ],
    )?;

    assert!(writer
        .update_chat_members(chat.topic_id, Vec::new())
        .is_err());

    assert!(writer
        .update_chat_members(TopicId::from_bytes([22; 32]), vec![member(alice, NwChatMemberStatus::Joined)])
        .is_err());

    let reader = reader_from(writer.conn);
    let updated_chat = reader.get_nw_chat(&chat.topic_id)?;

    assert_eq!(updated_chat.members.len(), 2);

    let mut updated_members = member_keys(&updated_chat);
    updated_members.sort_by(|left, right| left.0.cmp(&right.0));

    let mut expected_members = vec![
        (bob_endpoint.as_bytes().to_vec(), NwChatMemberStatus::Pending),
        (alice_endpoint.as_bytes().to_vec(), NwChatMemberStatus::Joined),
    ];
    expected_members.sort_by(|left, right| left.0.cmp(&right.0));

    assert_eq!(updated_members, expected_members);

    Ok(())
}

#[test]
fn profile_write_rejects_preexisting_profile_row() -> Result<()> {
    let writer = new_writer();
    let profile = profile();

    writer.conn.execute(
        "INSERT INTO user_profile (id, endpoint_id, secret_key, contact_name)
         VALUES (?1, ?2, ?3, ?4)",
        (
            1,
            profile.contact.endpoint_id.as_bytes(),
            profile.secret_key.to_bytes(),
            profile.contact.name.clone(),
        ),
    )?;

    assert!(writer.set_nw_profile(profile.clone()).is_err());

    let reader = reader_from(writer.conn);
    assert!(reader.get_nw_contact(&profile.contact.endpoint_id).is_err());

    Ok(())
}

#[test]
fn update_contact_name_is_idempotent() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");

    writer.add_nw_contact(alice.clone())?;

    writer.update_contact_name(alice.endpoint_id, "Alice Renamed".to_string())?;
    writer.update_contact_name(alice.endpoint_id, "Alice Renamed".to_string())?;

    let reader = reader_from(writer.conn);
    let updated = reader.get_nw_contact(&alice.endpoint_id)?;

    assert_eq!(updated.name, "Alice Renamed");

    Ok(())
}

#[test]
fn update_chat_members_is_atomic_when_fk_fails() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    let missing = contact(3, "Missing");

    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        23,
        vec![member(alice.clone(), NwChatMemberStatus::Pending)],
    );

    writer.add_nw_chat(chat.clone())?;

    assert!(writer
        .update_chat_members(
            chat.topic_id,
            vec![
                member(alice.clone(), NwChatMemberStatus::Joined),
                member(missing, NwChatMemberStatus::Pending),
            ],
        )
        .is_err());

    let reader = reader_from(writer.conn);
    let stored_chat = reader.get_nw_chat(&chat.topic_id)?;

    assert_eq!(member_keys(&stored_chat), member_keys(&chat));

    Ok(())
}