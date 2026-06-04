
use tokio::{runtime::Runtime};
use std::result::Result;
// use crate::nw::network_engine;
use crate::nw::NwDbManager;
use crate::ui::UiDbError;
use std::path::PathBuf;
use crate::notifications::{UiEventListener};
use crate::db::SCHEMA;


fn parse_path(path_str: &str) -> PathBuf {
    // This is completely infallible and converts the &str directly into a PathBuf
    PathBuf::from(path_str)
}



#[uniffi::export]
pub fn reset_db_for_wal(db_path_string: String) -> Result<(), UiDbError> {
    let db_path = parse_path(&db_path_string);
    
    if db_path.exists() {
        std::fs::remove_file(&db_path).map_err(|e| UiDbError::InternalError {
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

    let connection = rusqlite::Connection::open(&db_path).map_err(|e| UiDbError::InternalError {
        msg: format!("Failed to create empty database: {}", e),
    })?;

    let _: String = connection
        .query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
        .map_err(|e| UiDbError::InternalError {
            msg: format!("Failed to set WAL mode: {}", e),
        })?;

    // Execute the schema batch directly on the fresh connection
    connection.execute_batch(SCHEMA).map_err(|e| {
        log::error!("[DB] schema failed: {}", e);
        UiDbError::InternalError {
            msg: format!("Schema execution failed: {}", e),
        }
    })?;

    Ok(())
}


#[uniffi::export]
pub fn add_sample_messages(
    db_path_string: String,
    listener: Box<dyn UiEventListener>,
) -> Result<(), UiDbError> {
    let manager = NwDbManager::spawn(parse_path(&db_path_string))?;
    let client = manager.create_client(listener);

    let rt = Runtime::new()
        .map_err(|e| UiDbError::InternalError { msg: e.to_string() })?;

    rt.block_on(async move {
        client.add_sample_chat().await
    }).map_err(|e| UiDbError::InternalError { 
        msg: format!("Failed to add sample chat: {}", e) 
    })?;

    Ok(())
}
#[uniffi::export]
pub fn add_sample_message(
    db_path_string: String,
    listener: Box<dyn UiEventListener>,
    time: i32,
) -> Result<(), UiDbError> {
    let manager = NwDbManager::spawn(parse_path(&db_path_string))?;

    let client = manager.create_client(listener);

    let rt = Runtime::new()
        .map_err(|e| UiDbError::InternalError { msg: e.to_string() })?;

    rt.block_on(async move {
        client.add_sample_message(time).await
    }).map_err(|e| UiDbError::InternalError { 
        msg: format!("Failed to add sample message: {}", e) 
    })?;

    Ok(())
}

