//! test file for database UI reads
//! All tests passing proves:
//! 1) Empty UI reads are safe: with no data, contacts/chat headers are empty and profile is None.
//! 2) Invalid topic id strings are rejected for UI chat member/message/header lookups.
//! 3) UI chat messages are returned in chronological display order (earlier first).
//! 4) UI "last message" lookup returns the newest message when messages exist.
//! 5) UI chat headers are returned in deterministic topic order.
//! 6) UI sender mapping works for both local and remote identities (`Me` vs `Other`).
//! 7) Unknown senders do not break UI message reads; messages are still readable and classified as
//!    `Other`.
//! 8) UI contact lists are sorted predictably (name order as asserted by tests).
//! 9) Profile decoding for UI is correct: stored profile name and endpoint id are exposed exactly.
//! 10) UI chat header decoding is correct: topic/name/member count and member endpoint ids match
//!     stored network data.
//! 11) UI chat data aggregation is consistent with header views (same topic/name/member shape).
//! 12) End-to-end write->read behavior for UI projections is stable for contacts, chats, headers,
//!     messages, and profile fields.

use anyhow::Result;
use iroh_gossip::proto::TopicId;

use super::test_utils::*;
use crate::network::NwChatMemberStatus;

#[test]
fn empty_ui_reads_and_invalid_ids_return_expected_results() -> Result<()> {
    let reader = reader_from(connection());

    assert!(reader.get_ui_contacts()?.is_empty());
    assert!(reader.get_ui_chat_headers()?.is_empty());
    assert!(reader.get_ui_profile()?.is_none());

    assert!(reader.get_ui_chat_members("not-hex").is_err());
    assert!(reader.get_ui_chat_messages("not-hex").is_err());
    assert!(reader.get_ui_last_chat_message("not-hex").is_err());
    assert!(reader.get_ui_chat_header(&"00".repeat(32)).is_err());

    Ok(())
}

#[test]
fn ui_chat_messages_are_ordered_and_last_message_is_optional() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(profile.contact.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        13,
        vec![
            member(profile.contact.clone(), NwChatMemberStatus::Joined),
            member(alice.clone(), NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;

    writer.add_nw_message(message(chat.topic_id, alice.endpoint_id, "later", 20))?;
    writer.add_nw_message(message(chat.topic_id, alice.endpoint_id, "earlier", 10))?;

    let reader = reader_from(writer.conn);
    let ui_messages = reader.get_ui_chat_messages(&chat.topic_id.to_string())?;

    assert_eq!(ui_messages.len(), 2);
    assert_eq!(ui_messages[0].content, "earlier");
    assert_eq!(ui_messages[1].content, "later");

    assert_eq!(
        reader
            .get_ui_last_chat_message(&chat.topic_id.to_string())?
            .unwrap()
            .content,
        "later"
    );

    Ok(())
}

#[test]
fn ui_message_order_uses_row_id_when_timestamps_tie() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(profile.contact.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        27,
        vec![
            member(profile.contact.clone(), NwChatMemberStatus::Joined),
            member(alice.clone(), NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;

    writer.add_nw_message(message(chat.topic_id, alice.endpoint_id, "first", 100))?;
    writer.add_nw_message(message(chat.topic_id, alice.endpoint_id, "second", 100))?;

    let reader = reader_from(writer.conn);
    let messages = reader.get_ui_chat_messages(&chat.topic_id.to_string())?;

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "first");
    assert_eq!(messages[1].content, "second");

    let last = reader
        .get_ui_last_chat_message(&chat.topic_id.to_string())?
        .expect("last message should exist");

    assert_eq!(last.content, "second");

    Ok(())
}

#[test]
fn ui_chat_headers_are_deterministically_ordered() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile)?;
    writer.add_nw_contact(alice.clone())?;

    writer.add_nw_chat(chat(
        20,
        vec![member(alice.clone(), NwChatMemberStatus::Joined)],
    ))?;

    writer.add_nw_chat(chat(
        19,
        vec![member(alice, NwChatMemberStatus::Joined)],
    ))?;

    let reader = reader_from(writer.conn);
    let headers = reader.get_ui_chat_headers()?;

    assert_eq!(
        headers[0].topic_id,
        TopicId::from_bytes([19; 32]).to_string()
    );
    assert_eq!(
        headers[1].topic_id,
        TopicId::from_bytes([20; 32]).to_string()
    );

    Ok(())
}

#[test]
fn ui_message_reads_cover_me_unknown_senders_and_all_messages() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");
    let unknown = contact(99, "Unknown");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(profile.contact.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        15,
        vec![
            member(profile.contact.clone(), NwChatMemberStatus::Joined),
            member(alice, NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;

    writer.add_nw_message(message(
        chat.topic_id,
        profile.contact.endpoint_id,
        "mine",
        1,
    ))?;

    writer.add_nw_message(message(
        chat.topic_id,
        unknown.endpoint_id,
        "unknown",
        2,
    ))?;

    let reader = reader_from(writer.conn);
    let ui_messages = reader.get_ui_chat_messages(&chat.topic_id.to_string())?;

    assert_eq!(ui_messages.len(), 2);
    assert_eq!(ui_messages[0].content, "mine");
    assert_eq!(ui_messages[1].content, "unknown");

    assert!(matches!(
        ui_messages[0].sender,
        crate::ui::UiSender::Me
    ));

    assert!(matches!(
        ui_messages[1].sender,
        crate::ui::UiSender::Other(_)
    ));

    Ok(())
}

#[test]
fn contacts_are_sorted_and_unknown_ui_senders_are_supported() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let zed = contact(1, "Zed");
    let alice = contact(2, "Alice");

    writer.set_nw_profile(profile)?;
    writer.add_nw_contact(zed.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        16,
        vec![
            member(zed, NwChatMemberStatus::Joined),
            member(alice, NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;

    let unknown = contact(99, "Unknown");

    writer.add_nw_message(message(
        chat.topic_id,
        unknown.endpoint_id,
        "unknown",
        1,
    ))?;

    let reader = reader_from(writer.conn);
    let contacts = reader.get_ui_contacts()?;

    assert_eq!(contacts.len(), 3);
    assert_eq!(contacts[0].name, "Alice");
    assert_eq!(contacts[1].name, "Me");
    assert_eq!(contacts[2].name, "Zed");

    assert_eq!(
        reader
            .get_ui_last_chat_message(&chat.topic_id.to_string())?
            .unwrap()
            .content,
        "unknown"
    );

    Ok(())
}

#[test]
fn ui_reads_decode_contacts_profiles_and_chat_data() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(profile.contact.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        14,
        vec![
            member(profile.contact.clone(), NwChatMemberStatus::Joined),
            member(alice, NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;

    let reader = reader_from(writer.conn);

    let ui_profile = reader.get_ui_profile()?.unwrap();

    assert_eq!(ui_profile.name, profile.contact.name);
    assert_eq!(
        ui_profile.endpoint_id,
        hex::encode(profile.contact.endpoint_id.as_bytes())
    );

    let header = reader.get_ui_chat_header(&chat.topic_id.to_string())?;

    assert_eq!(header.name, chat.name);
    assert_eq!(header.members.len(), 2);

    assert!(
        header
            .members
            .iter()
            .any(|member| {
                member.endpoint_id
                    == hex::encode(profile.contact.endpoint_id.as_bytes())
            })
    );

    assert_eq!(reader.get_ui_chat_headers()?.len(), 1);

    let data = reader.get_ui_chat_data(&chat.topic_id.to_string())?;

    assert_eq!(data.chat.name, header.name);
    assert_eq!(data.chat.members.len(), header.members.len());
    assert_eq!(data.chat.topic_id, header.topic_id);

    Ok(())
}

#[test]
fn ui_chat_header_has_no_last_message_when_chat_is_empty() -> Result<()> {
    let mut writer = new_writer();
    let profile = profile();
    let alice = contact(1, "Alice");

    writer.set_nw_profile(profile.clone())?;
    writer.add_nw_contact(profile.contact.clone())?;
    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        25,
        vec![
            member(profile.contact, NwChatMemberStatus::Joined),
            member(alice, NwChatMemberStatus::Joined),
        ],
    );

    writer.add_nw_chat(chat.clone())?;

    let reader = reader_from(writer.conn);
    let header = reader.get_ui_chat_header(&chat.topic_id.to_string())?;

    assert!(header.last_message.is_none());
    assert!(reader
        .get_ui_last_chat_message(&chat.topic_id.to_string())?
        .is_none());

    Ok(())
}

#[test]
fn ui_message_reads_fail_when_profile_is_missing() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");

    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        26,
        vec![member(alice.clone(), NwChatMemberStatus::Joined)],
    );

    writer.add_nw_chat(chat.clone())?;
    writer.add_nw_message(message(chat.topic_id, alice.endpoint_id, "hello", 1))?;

    let reader = reader_from(writer.conn);

    assert!(reader.get_ui_chat_messages(&chat.topic_id.to_string()).is_err());
    assert!(reader
        .get_ui_last_chat_message(&chat.topic_id.to_string())
        .is_err());
    assert!(reader.get_ui_chat_header(&chat.topic_id.to_string()).is_err());

    Ok(())
}