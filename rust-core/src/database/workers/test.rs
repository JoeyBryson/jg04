use anyhow::Result;
use iroh::{EndpointId, SecretKey};
use iroh_gossip::proto::TopicId;
use rusqlite::Connection;
use tokio::sync::mpsc as tokio_mpsc;

use super::{DbReader, DbWriter};
use crate::network::{NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile};

fn connection() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../sql/schema.sql"))
        .unwrap();
    connection
}

fn new_writer() -> DbWriter {
    let (_, receiver) = tokio_mpsc::channel(1);
    DbWriter {
        rx: receiver,
        conn: connection(),
    }
}

fn reader_from(connection: Connection) -> DbReader {
    let (_, receiver) = tokio_mpsc::channel(1);
    DbReader {
        rx: receiver,
        conn: connection,
    }
}

fn contact(seed: u8, name: &str) -> NwContact {
    NwContact {
        name: name.to_string(),
        endpoint_id: EndpointId::from(SecretKey::from_bytes(&[seed; 32]).public()),
    }
}

fn profile() -> NwProfile {
    let secret_key = SecretKey::from_bytes(&[42; 32]);
    NwProfile {
        contact: NwContact {
            name: "Me".to_string(),
            endpoint_id: EndpointId::from(secret_key.public()),
        },
        secret_key,
    }
}

fn chat(topic_seed: u8, members: Vec<NwChatMember>) -> NwChat {
    NwChat {
        name: Some("Test chat".to_string()),
        members,
        topic_id: TopicId::from_bytes([topic_seed; 32]),
    }
}

fn member(contact: NwContact, status: NwChatMemberStatus) -> NwChatMember {
    NwChatMember { contact, status }
}

fn message(topic_id: TopicId, content: &str, sent_at: i64) -> NwMessage {
    NwMessage {
        topic_id,
        from_me: false,
        endpoint_id: Some(contact(1, "Alice").endpoint_id),
        content: content.to_string(),
        sent_at,
    }
}

fn member_keys(chat: &NwChat) -> Vec<(Vec<u8>, NwChatMemberStatus)> {
    member_keys_from_members(&chat.members)
}

fn member_keys_from_members(members: &[NwChatMember]) -> Vec<(Vec<u8>, NwChatMemberStatus)> {
    let mut members = members
        .iter()
        .map(|member| {
            (
                member.contact.endpoint_id.as_bytes().to_vec(),
                member.status,
            )
        })
        .collect::<Vec<_>>();
    members.sort_by(|left, right| left.0.cmp(&right.0));
    members
}

#[test]
fn profile_write_is_idempotent_and_conflicts_are_rejected() -> Result<()> {
    let writer = new_writer();
    let profile = profile();

    writer.set_nw_profile(profile.clone())?;
    writer.set_nw_profile(profile.clone())?;

    let mut conflicting = profile.clone();
    conflicting.contact.name = "Other".to_string();
    assert!(writer.set_nw_profile(conflicting).is_err());

    let reader = reader_from(writer.conn);
    let stored = reader.get_nw_profile()?;
    assert_eq!(stored.secret_key.to_bytes(), profile.secret_key.to_bytes());
    assert_eq!(stored.contact, profile.contact);
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
    assert_eq!(reader.get_ui_contacts()?.len(), 1);
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
    assert_eq!(
        member_keys_from_members(&reader.get_nw_chat_members(&chat.topic_id)?),
        member_keys(&chat)
    );
    let stored_chats = reader.get_nw_chats()?;
    assert_eq!(stored_chats.len(), 1);
    assert_eq!(member_keys(&stored_chats[0]), member_keys(&chat));

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
        "INSERT INTO contacts (endpoint_id, contact_name) VALUES (?1, ?2)",
        (&malformed_endpoint, "Malformed"),
    )?;
    writer.conn.execute(
        "INSERT INTO chats (topic_id, chat_name) VALUES (?1, ?2)",
        (malformed_topic.as_bytes(), "Malformed"),
    )?;
    writer.conn.execute(
        "INSERT INTO chat_members (topic_id, endpoint_id, status) VALUES (?1, ?2, ?3)",
        (malformed_topic.as_bytes(), &malformed_endpoint, 0),
    )?;

    let reader = reader_from(writer.conn);
    assert!(reader.get_nw_chats().is_err());
    Ok(())
}

#[test]
fn empty_reads_and_invalid_ui_ids_return_expected_results() -> Result<()> {
    let reader = reader_from(connection());
    assert!(reader.get_nw_chats()?.is_empty());
    assert!(reader.get_nw_messages()?.is_empty());
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
fn marking_members_joined_is_idempotent_and_missing_rows_fail() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    writer.add_nw_contact(alice.clone())?;
    let chat = chat(11, vec![member(alice.clone(), NwChatMemberStatus::Pending)]);
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
fn messages_are_read_in_ui_order_and_last_message_is_optional() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    writer.add_nw_contact(alice.clone())?;
    let chat = chat(13, vec![member(alice, NwChatMemberStatus::Joined)]);
    writer.add_nw_chat(chat.clone())?;

    writer.add_nw_message(message(chat.topic_id, "later", 20))?;
    writer.add_nw_message(message(chat.topic_id, "earlier", 10))?;

    let reader = reader_from(writer.conn);
    let messages = reader.get_nw_chat_messages(&chat.topic_id)?;
    assert_eq!(messages[0].content, "later");
    assert_eq!(messages[1].content, "earlier");

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
fn network_chat_and_header_reads_are_deterministically_ordered() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    writer.add_nw_contact(alice.clone())?;
    writer.add_nw_chat(chat(
        20,
        vec![member(alice.clone(), NwChatMemberStatus::Joined)],
    ))?;
    writer.add_nw_chat(chat(19, vec![member(alice, NwChatMemberStatus::Joined)]))?;

    let reader = reader_from(writer.conn);
    let chats = reader.get_nw_chats()?;
    assert_eq!(chats[0].topic_id, TopicId::from_bytes([19; 32]));
    assert_eq!(chats[1].topic_id, TopicId::from_bytes([20; 32]));

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
fn message_reads_cover_from_me_missing_senders_and_all_messages() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    writer.add_nw_contact(alice.clone())?;
    let chat = chat(15, vec![member(alice, NwChatMemberStatus::Joined)]);
    writer.add_nw_chat(chat.clone())?;

    writer.add_nw_message(NwMessage {
        topic_id: chat.topic_id,
        from_me: true,
        endpoint_id: None,
        content: "mine".to_string(),
        sent_at: 1,
    })?;
    writer.add_nw_message(NwMessage {
        topic_id: chat.topic_id,
        from_me: false,
        endpoint_id: None,
        content: "unknown".to_string(),
        sent_at: 2,
    })?;

    let reader = reader_from(writer.conn);
    let messages = reader.get_nw_messages()?;
    assert_eq!(messages.len(), 2);
    assert!(messages.iter().any(|message| message.from_me));
    assert!(messages.iter().any(|message| message.endpoint_id.is_none()));

    let ui_messages = reader.get_ui_chat_messages(&chat.topic_id.to_string())?;
    assert_eq!(ui_messages.len(), 2);
    assert_eq!(ui_messages[0].content, "mine");
    assert_eq!(ui_messages[1].content, "unknown");
    Ok(())
}

#[test]
fn contacts_are_sorted_and_unknown_ui_senders_are_supported() -> Result<()> {
    let mut writer = new_writer();
    let zed = contact(1, "Zed");
    let alice = contact(2, "Alice");
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
    writer.add_nw_message(NwMessage {
        topic_id: chat.topic_id,
        from_me: false,
        endpoint_id: None,
        content: "unknown".to_string(),
        sent_at: 1,
    })?;

    let reader = reader_from(writer.conn);
    let contacts = reader.get_ui_contacts()?;
    assert_eq!(contacts[0].name, "Alice");
    assert_eq!(contacts[1].name, "Zed");
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
    writer.add_nw_contact(alice.clone())?;
    let chat = chat(14, vec![member(alice, NwChatMemberStatus::Joined)]);
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
    assert_eq!(header.members.len(), 1);
    assert_eq!(reader.get_ui_chat_headers()?.len(), 1);
    let data = reader.get_ui_chat_data(&chat.topic_id.to_string())?;
    assert_eq!(data.chat.name, header.name);
    assert_eq!(data.chat.members.len(), header.members.len());
    assert_eq!(data.chat.topic_id, header.topic_id);
    Ok(())
}

#[test]
fn convenience_writes_and_chat_conflicts_are_handled() -> Result<()> {
    let mut writer = new_writer();
    let alice = contact(1, "Alice");
    writer.add_ui_contact(crate::ui::UiContact {
        name: alice.name.clone(),
        endpoint_id: hex::encode(alice.endpoint_id.as_bytes()),
    })?;

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
        members: vec![member(alice, NwChatMemberStatus::Pending)],
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
    let alice = contact(1, "Alice");
    writer.set_nw_profile(profile())?;
    writer.add_nw_contact(alice.clone())?;
    writer.add_nw_chat(chat(17, vec![member(alice, NwChatMemberStatus::Joined)]))?;
    writer.reset_database()?;
    writer.reset_database()?;

    let reader = reader_from(writer.conn);
    assert!(reader.get_nw_profile().is_err());
    assert!(reader.get_ui_profile()?.is_none());
    assert!(reader.get_nw_chats()?.is_empty());
    assert!(reader.get_ui_contacts()?.is_empty());
    Ok(())
}
