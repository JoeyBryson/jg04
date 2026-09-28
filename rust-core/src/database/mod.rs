//! The database module of ___-lib.
//!
//! This is the bottom layer of the crate and the source of truth for
//! application state during runtime.
//!
//! Database calls can be made from anywhere through the [`DbClient`] API.
//!
//! ```text
//! db_client.add_nw_chat(chat);
//! let chats: Vec<NwChat> = db_client.get_nw_chats();
//! ```
//!
//! [`DbClient`] instances are spawned from the [`DbManager`], which
//! manages database workers and connections.
//!
//! ```text
//! let db_manager = DbManager::spawn(db_path_string);
//! let db_client = db_manager.spawn_client();
//! ```
//! 
//! The database layer should remain independent of higher-level
//! application logic, but it does take and return primative types 
//! from other dependencies needed to represent Ui or Networking state strictly through the
//! methods of DbClient.


use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use tokio::sync::mpsc;

use crate::ffi_error::FfiError;

pub mod client;
#[cfg(test)]
mod client_test;
mod macros;
pub mod manager;
pub mod sample_data_insertions;
mod workers;


//Run this to reset the db after schema changes. 
//Once we have real users database migrations will have to be implemented
#[uniffi::export]
pub fn delete_db(db_path_string: String) -> Result<(), FfiError> {
    (move || -> anyhow::Result<()> {
        let db_path = PathBuf::from_str(&db_path_string)?;

        if db_path.exists() {
            std::fs::remove_file(&db_path)?;
        }

        log::info!("[DB] database file deleted");

        Ok(())
    })()
    .map_err(FfiError::from)
}