
use iroh::{EndpointId, endpoint};
use tokio::{runtime::Runtime};
use std::os::unix::process;
use std::result::Result;
// use crate::nw::network_engine;
use crate::database::manager::DbManager;
use crate::ffi_error::FfiError;
use crate::network::{NwChat, NwContact};
use crate::ui::UiContact;
use std::path::PathBuf;
use std::sync::Arc;
use crate::database::client::DbClient;
use iroh_gossip::TopicId;

pub const SCHEMA: &str = include_str!("../sql/schema.sql");

pub fn parse_path(path_str: &str) -> PathBuf {
    // This is completely infallible and converts the &str directly into a PathBuf
    PathBuf::from(path_str)
}



#[uniffi::export]
pub fn reset_db_for_wal(db_path_string: String) -> Result<(), FfiError> {
    let db_path = parse_path(&db_path_string);
    
    if db_path.exists() {
        std::fs::remove_file(&db_path).map_err(|e| FfiError::Internal {
            msg: format!("Failed to delete db file: {}", e),
        })?;
    }

    let wal_path = parse_path(&format!("{}-wal", db_path_string));
    if wal_path.exists() {
        let _ = std::fs::remove_file(&wal_path);
    }

    let shm_path = parse_path(&format!("{}-shm", db_path_string));
    if shm_path.exists() {
        let _ = std::fs::remove_file(&shm_path);
    }

    let connection = rusqlite::Connection::open(&db_path).map_err(|e| FfiError::Internal {
        msg: format!("Failed to create empty database: {}", e),
    })?;

    let _: String = connection
        .query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
        .map_err(|e| FfiError::Internal {
            msg: format!("Failed to set WAL mode: {}", e),
        })?;

    // Execute the schema batch directly on the fresh connection
    connection.execute_batch(SCHEMA).map_err(|e| {
        log::error!("[DB] schema failed: {}", e);
        FfiError::Internal {
            msg: format!("Schema execution failed: {}", e),
        }
    })?;

    Ok(())
}


#[uniffi::export]
pub fn add_sample_messages(
    db_path_string: String,
) -> Result<(), FfiError> {
    let manager = DbManager::spawn(db_path_string)?;
    let client = manager.spawn_client();

    let rt = Runtime::new()
        .map_err(|e| FfiError::Internal { msg: e.to_string() })?;

    rt.block_on(async move {
        client.add_sample_chat().await
    }).map_err(|e| FfiError::Internal { 
        msg: format!("Failed to add sample chat: {}", e) 
    })?;

    Ok(())
}

#[uniffi::export]
pub fn add_sample_message(
    client: Arc<DbClient>,
    time: i32,
) -> Result<(), FfiError> {

    let rt = Runtime::new()
        .map_err(|e| FfiError::Internal { msg: e.to_string() })?;

    rt.block_on(async move {
        client.add_sample_message(time).await
    }).map_err(|e| FfiError::Internal { 
        msg: format!("Failed to add sample message: {}", e) 
    })?;

    Ok(())
}

#[uniffi::export]
pub fn add_sample_data(
    db_path_string: String,
) -> Result<(), FfiError> {

    let manager =
        DbManager::spawn(db_path_string)?;

    let client = manager.spawn_client();

    let rt = Runtime::new()
        .map_err(|e| FfiError::Internal {
            msg: e.to_string()
        })?;

    rt.block_on(async move {
        client.add_sample_data().await
    })
    .map_err(|e| FfiError::Internal {
        msg: format!(
            "Failed to add sample data: {}",
            e
        )
    })?;

    Ok(())
}


#[uniffi::export]
fn print_endpoint_id(db_client: Arc<DbClient>) -> std::result::Result<(), FfiError> {
    let profile = db_client.get_nw_profile()?;
    
    let endpoint_id_hex = hex::encode(profile.secret_key.public().as_bytes());
    log::info!("Endpoint_id: {}", endpoint_id_hex);
    
    Ok(())
}

#[uniffi::export]
fn add_contact_id(
    db_client: Arc<DbClient>, 
    name: String, 
    endpoint_id_hex: String
) -> std::result::Result<(), FfiError> {
    let bytes: [u8; 32] = hex::decode(&endpoint_id_hex)
        .map_err(anyhow::Error::from)?      
        .try_into()                         
        .map_err(|_| anyhow::anyhow!("Endpoint ID must be exactly 32 bytes"))?;

    let endpoint_id = EndpointId::from_bytes(&bytes)
        .map_err(anyhow::Error::from)? ;

    let contact = NwContact {
        name,
        endpoint_id,
    };

    db_client.add_nw_contact_sync(contact)?;

    Ok(())
}


#[uniffi::export]
fn add_chat(
    db_client: Arc<DbClient>,
    chat_name: String,
    endpoint_ids_hex: Vec<String>,
    topic_id_hex: String,
) -> std::result::Result<(), FfiError> {
    let contacts = db_client.get_ui_contacts()?;
    let mut members = Vec::with_capacity(endpoint_ids_hex.len());

    for hex_str in &endpoint_ids_hex {
        let contact = contacts
            .iter()
            .find(|c| &c.endpoint_id == hex_str)
            .ok_or_else(|| anyhow::anyhow!("Contact not found for endpoint ID: {}", hex_str))?;

        // 2. Decode endpoint ID bytes
        let bytes: [u8; 32] = hex::decode(hex_str)
            .map_err(anyhow::Error::from)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Endpoint ID must be exactly 32 bytes"))?;

        let endpoint_id = EndpointId::from_bytes(&bytes)
            .map_err(anyhow::Error::from)?;

        let member = NwContact {
            name: contact.name.clone(),
            endpoint_id,
        };

        members.push(member);
    }

    let topic_bytes: [u8; 32] = hex::decode(&topic_id_hex)
        .map_err(anyhow::Error::from)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Topic ID must be exactly 32 bytes"))?;

    let topic_id = TopicId::from_bytes(topic_bytes);

    let chat = NwChat {
        name: Some(chat_name),
        members,
        topic_id,
    };

    db_client.add_nw_chat_sync(chat)?;

    Ok(())
}