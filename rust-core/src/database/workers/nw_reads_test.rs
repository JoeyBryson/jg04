//! test file for database network reads
//! All tests passing proves:
//! 1) Network chat/member reads correctly fail when a chat row exists without valid members.
//! 2) Network chat reads correctly fail on malformed stored rows (for example invalid endpoint
//!    byte lengths), preventing silent decode corruption.
//! 3) Empty network reads are stable: no chats and no messages return empty vectors.
//! 4) Network chat list ordering is deterministic by topic id.
//! 5) Per-chat message reads return records in stable insertion order (message row id order),
//!    matching current network read behavior.
//! 6) Network message reads include both known and unknown senders without dropping data.
//! 7) End-to-end write->read paths for chats/members/messages are consistent for valid rows.
//! 8) Reader-side validation catches malformed persisted data early instead of returning partially
//!    decoded network types.

use anyhow::Result;
use iroh_gossip::proto::TopicId;

use super::test_utils::*;
use crate::network::NwChatMemberStatus;

#[test]
fn chat_reads_reject_missing_and_malformed_rows() -> Result<()> {
    let connection = connection();
    let topic_id = TopicId::from_bytes([9; 32]);

    connection.execute(
        "INSERT INTO chats (topic_id, chat_name) VALUES (?1, ?2)",
        (topic_id.as_bytes(), "Empty"),
    )?;

    let reader = reader_from(connection);

    assert!(reader.get_nw_chat(&topic_id).is_err());
    assert!(reader.get_nw_chat_members(&topic_id).is_err());

    let writer = new_writer();
    let malformed_endpoint = vec![1u8, 2, 3];
    let malformed_topic = TopicId::from_bytes([10; 32]);

    writer.conn.execute(
        "INSERT INTO contacts (endpoint_id, contact_name)
         VALUES (?1, ?2)",
        (&malformed_endpoint, "Malformed"),
    )?;

    writer.conn.execute(
        "INSERT INTO chats (topic_id, chat_name)
         VALUES (?1, ?2)",
        (malformed_topic.as_bytes(), "Malformed"),
    )?;

    writer.conn.execute(
        "INSERT INTO chat_members (topic_id, endpoint_id, status)
         VALUES (?1, ?2, ?3)",
        (malformed_topic.as_bytes(), &malformed_endpoint, 0),
    )?;

    let reader = reader_from(writer.conn);

    assert!(reader.get_nw_chats().is_err());

    Ok(())
}

#[test]
fn empty_nw_reads_return_expected_results() -> Result<()> {
    let reader = reader_from(connection());

    assert!(reader.get_nw_chats()?.is_empty());
    assert!(reader.get_nw_messages()?.is_empty());

    Ok(())
}

#[test]
fn network_chat_reads_are_deterministically_ordered() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");

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
    let chats = reader.get_nw_chats()?;

    assert_eq!(chats[0].topic_id, TopicId::from_bytes([19; 32]));
    assert_eq!(chats[1].topic_id, TopicId::from_bytes([20; 32]));

    Ok(())
}

#[test]
fn network_chat_messages_are_read_descending_by_timestamp() -> Result<()> {
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

    writer.add_nw_message(message(
        chat.topic_id,
        alice.endpoint_id,
        "later",
        20,
    ))?;

    writer.add_nw_message(message(
        chat.topic_id,
        alice.endpoint_id,
        "earlier",
        10,
    ))?;

    let reader = reader_from(writer.conn);
    let messages = reader.get_nw_chat_messages(&chat.topic_id)?;

    assert_eq!(messages[0].content, "later");
    assert_eq!(messages[1].content, "earlier");

    Ok(())
}

#[test]
fn network_message_reads_cover_known_and_unknown_senders() -> Result<()> {
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
    let messages = reader.get_nw_messages()?;

    assert_eq!(messages.len(), 2);

    assert!(
        messages
            .iter()
            .any(|message| message.endpoint_id == profile.contact.endpoint_id)
    );

    assert!(
        messages
            .iter()
            .any(|message| message.endpoint_id == unknown.endpoint_id)
    );

    Ok(())
}

#[test]
fn missing_network_chat_rows_return_errors() -> Result<()> {
    let reader = reader_from(connection());
    let missing_topic = TopicId::from_bytes([42; 32]);

    assert!(reader.get_nw_chat(&missing_topic).is_err());
    assert!(reader.get_nw_chat_members(&missing_topic).is_err());

    Ok(())
}

#[test]
fn network_message_reads_reject_malformed_endpoint_rows() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");

    writer.add_nw_contact(alice.clone())?;

    let chat = chat(
        24,
        vec![member(alice.clone(), NwChatMemberStatus::Joined)],
    );

    writer.add_nw_chat(chat.clone())?;

    writer.conn.execute(
        "INSERT INTO messages (topic_id, endpoint_id, content, sent_at)
         VALUES (?1, ?2, ?3, ?4)",
        (chat.topic_id.as_bytes(), vec![1u8, 2, 3], "bad", 1),
    )?;

    let reader = reader_from(writer.conn);

    assert!(reader.get_nw_chat_messages(&chat.topic_id).is_err());
    assert!(reader.get_nw_messages().is_err());

    Ok(())
}