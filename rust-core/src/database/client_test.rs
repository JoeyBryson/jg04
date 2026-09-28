use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use iroh::{EndpointId, SecretKey};
use iroh_gossip::TopicId;
use tokio::sync::mpsc as tokio_mpsc;

use super::manager::{DbManager, DbMode};
use super::workers::DbReader;
use crate::network::{NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage};
use crate::ui::UiContact;

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_nanos();

    std::env::temp_dir().join(format!("rust_api_dbclient_test_{}_{}.db", std::process::id(), nanos))
}

fn new_direct_reader(db_path: &Path) -> Result<DbReader> {
    let (_, rx) = tokio_mpsc::channel(1);
    let conn = DbManager::start_conn(db_path, DbMode::ReadOnly)?;

    DbReader::new(rx, conn)
}

#[test]
fn dbclient_get_ui_contacts_matches_direct_reader() -> Result<()> {
    let db_path = temp_db_path();
    let manager = DbManager::spawn(db_path.to_string_lossy().into_owned())?;
    let client = manager.spawn_client_raw();

    let alice = NwContact {
        name: "Alice".to_string(),
        endpoint_id: EndpointId::from(SecretKey::from_bytes(&[1; 32]).public()),
    };
    let bob = NwContact {
        name: "Bob".to_string(),
        endpoint_id: EndpointId::from(SecretKey::from_bytes(&[2; 32]).public()),
    };

    client.add_ui_contact(UiContact {
        name: alice.name.clone(),
        endpoint_id: hex::encode(alice.endpoint_id.as_bytes()),
    })?;
    client.add_ui_contact(UiContact {
        name: bob.name.clone(),
        endpoint_id: hex::encode(bob.endpoint_id.as_bytes()),
    })?;

    let via_client = client.get_ui_contacts()?;

    let direct_reader = new_direct_reader(&db_path)?;
    let via_worker = direct_reader.get_ui_contacts()?;

    assert_eq!(via_client, via_worker);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[test]
fn dbclient_get_nw_chats_sync_matches_direct_reader() -> Result<()> {
    let db_path = temp_db_path();
    let manager = DbManager::spawn(db_path.to_string_lossy().into_owned())?;
    let client = manager.spawn_client_raw();

    let alice = NwContact {
        name: "Alice".to_string(),
        endpoint_id: EndpointId::from(SecretKey::from_bytes(&[11; 32]).public()),
    };
    let bob = NwContact {
        name: "Bob".to_string(),
        endpoint_id: EndpointId::from(SecretKey::from_bytes(&[12; 32]).public()),
    };

    client.add_nw_contact_sync(alice.clone())?;
    client.add_nw_contact_sync(bob.clone())?;

    let inserted_chat = NwChat {
        name: Some("Test chat".to_string()),
        members: vec![
            NwChatMember {
                contact: alice,
                status: NwChatMemberStatus::Joined,
            },
            NwChatMember {
                contact: bob,
                status: NwChatMemberStatus::Pending,
            },
        ],
        topic_id: TopicId::from_bytes([7; 32]),
    };

    client.add_nw_chat_sync(inserted_chat.clone())?;

    let via_client = client.get_nw_chats_sync()?;

    let direct_reader = new_direct_reader(&db_path)?;
    let via_worker = direct_reader.get_nw_chats()?;

    assert_eq!(via_client, via_worker);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn dbclient_get_nw_chat_messages_matches_direct_reader() -> Result<()> {
    let db_path = temp_db_path();
    let manager = DbManager::spawn(db_path.to_string_lossy().into_owned())?;
    let client = manager.spawn_client_raw();

    let alice = NwContact {
        name: "Alice".to_string(),
        endpoint_id: EndpointId::from(SecretKey::from_bytes(&[21; 32]).public()),
    };
    client.add_nw_contact(alice.clone()).await?;

    let inserted_chat = NwChat {
        name: Some("Test chat".to_string()),
        members: vec![NwChatMember {
            contact: alice.clone(),
            status: NwChatMemberStatus::Joined,
        }],
        topic_id: TopicId::from_bytes([9; 32]),
    };
    client.add_nw_chat(inserted_chat.clone()).await?;

    client
        .add_nw_message(NwMessage {
            topic_id: inserted_chat.topic_id,
            from_me: false,
            endpoint_id: Some(alice.endpoint_id),
            content: "first".to_string(),
            sent_at: 1,
        })
        .await?;

    client
        .add_nw_message(NwMessage {
            topic_id: inserted_chat.topic_id,
            from_me: true,
            endpoint_id: None,
            content: "second".to_string(),
            sent_at: 2,
        })
        .await?;

    let via_client = client.get_nw_chat_messages(inserted_chat.topic_id).await?;

    let direct_reader = new_direct_reader(&db_path)?;
    let via_worker = direct_reader.get_nw_chat_messages(&inserted_chat.topic_id)?;

    assert_eq!(via_client, via_worker);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn sync_read_methods_return_error_in_async_runtime() -> Result<()> {
    let db_path = temp_db_path();
    let manager = DbManager::spawn(db_path.to_string_lossy().into_owned())?;
    let client = manager.spawn_client_raw();

    let error = client
        .get_nw_chats_sync()
        .expect_err("sync read should fail inside async runtime");

    assert!(
        error
            .to_string()
            .contains("called from async runtime"),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn sync_write_methods_return_error_in_async_runtime() -> Result<()> {
    let db_path = temp_db_path();
    let manager = DbManager::spawn(db_path.to_string_lossy().into_owned())?;
    let client = manager.spawn_client_raw();

    let error = client
        .add_nw_contact_sync(NwContact {
            name: "AsyncMisuse".to_string(),
            endpoint_id: EndpointId::from(SecretKey::from_bytes(&[31; 32]).public()),
        })
        .expect_err("sync write should fail inside async runtime");

    assert!(
        error
            .to_string()
            .contains("called from async runtime"),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}
