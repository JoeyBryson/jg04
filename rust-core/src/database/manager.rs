use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use tokio::sync::mpsc;

use super::{
    client::{DbClient, ReadRequest, WriteRequest},
    workers::{DbReader, DbWriter},
};
use crate::ffi_error::FfiError;

#[derive(Debug)]
pub enum DbMode {
    ReadOnly,
    ReadWrite,
}
/// Manages the database and spawns [`DbClient`] instances.
///
/// [`DbManager`] spawns two database workers, [`DbReader`] and [`DbWriter`].
/// Each worker runs in its own [`std::thread`] and receives database requests
/// through a [`tokio::sync::mpsc::Receiver`].
///
/// Each [`DbClient`] is given a clone of the corresponding
/// [`tokio::sync::mpsc::Sender`]
///
/// A single [`DbManager`] should exist for each database. In the released
/// application, this means there is one [`DbManager`]. The test framework 
/// allows multiple nodes to be simulated, each node having its own database and [`DbManager`].
/// 
/// # Example
///
/// ```
/// # let db_path = std::env::temp_dir()
/// #    .join("rust_api_example.db")
/// #    .to_string_lossy()
/// #    .into_owned();
/// use rust_api::database::manager::DbManager;
///
/// let manager = DbManager::spawn(db_path.clone()).unwrap();
/// let _client = manager.spawn_client();
/// # std::fs::remove_file(db_path).ok();
/// ```
/// 
#[derive(uniffi::Object)]
pub struct DbManager {
    reader_tx: mpsc::Sender<ReadRequest>,
    writer_tx: mpsc::Sender<WriteRequest>,
    _reader_handle: Option<JoinHandle<()>>,
    _writer_handle: Option<JoinHandle<()>>,
}

impl DbManager {
    pub fn start_conn(db_path: &Path, mode: DbMode) -> Result<Connection> {

        let flags = match mode {
            DbMode::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
            DbMode::ReadWrite => {
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
            }
        };

        let conn = Connection::open_with_flags(db_path, flags).map_err(|e| {
            log::error!("[DB-MANAGER] open failed: {}", e);
            e
        })?;

        if matches!(mode, DbMode::ReadWrite) {
            let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");
        }

        Ok(conn)
    }

    fn execute_schema(db_path: &Path) -> Result<()> {

        let conn = Self::start_conn(db_path, DbMode::ReadWrite)?;
        conn.execute_batch(include_str!("sql/schema.sql"))?;

        log::info!("[DB-MANAGER] schema executed");

        Ok(())
    }

    pub(crate) fn spawn_workers(
        db_path: &PathBuf,
    ) -> (
        mpsc::Sender<ReadRequest>,
        mpsc::Sender<WriteRequest>,
        JoinHandle<()>,
        JoinHandle<()>,
    ) {
        let (reader_tx, reader_rx) = mpsc::channel::<ReadRequest>(32);
        let (writer_tx, writer_rx) = mpsc::channel::<WriteRequest>(32);

        let db_path_clone = db_path.clone();
        let writer_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let conn = Self::start_conn(&db_path_clone, DbMode::ReadWrite)?;
                let writer = DbWriter::new(writer_rx, conn)?;
                writer.request_loop();
                Ok(())
            })();

            if let Err(error) = result {
                log::error!("[DB-WRITER] exited with error: {}", error);
            } else {
                log::warn!("[DB-WRITER] exited without error");
            }
        });

        let db_path_clone = db_path.clone();
        let reader_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let conn = Self::start_conn(&db_path_clone, DbMode::ReadOnly)?;
                let reader = DbReader::new(reader_rx, conn)?;
                reader.request_loop();
                Ok(())
            })();

            if let Err(error) = result {
                log::error!("[DB-READER] exited with error: {}", error);
            } else {
                log::warn!("[DB-READER] exited without error");
            }
        });

        (reader_tx, writer_tx, reader_handle, writer_handle)
    }
}

#[uniffi::export]
impl DbManager {
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Self, FfiError> {
        (move || -> anyhow::Result<Self> {
            let db_path = PathBuf::from_str(&db_path_string)?;

            Self::execute_schema(&db_path)?;

            let (reader_tx, writer_tx, reader_handle, writer_handle) = Self::spawn_workers(&db_path);

            Ok(Self {
                reader_tx,
                writer_tx,
                _reader_handle: Some(reader_handle),
                _writer_handle: Some(writer_handle),
            })
        })()
        .map_err(FfiError::from)
    }

    //Arc is required by uniffi for memory safety across ffi
    //Note: uniffi constructors wrap their objects in Arc implicitly
    
    pub fn spawn_client(&self) -> Arc<DbClient> {
        Arc::new(DbClient::new(self.reader_tx.clone(), self.writer_tx.clone()))
    }
}


impl DbManager {
    //incase we want an unwrapped DbClient for in-crate purposes
    //may not be necessary since we can just clone DbClient to our hearts content
    pub fn spawn_client_raw(&self) -> DbClient {
        DbClient::new(self.reader_tx.clone(), self.writer_tx.clone())
    }
}