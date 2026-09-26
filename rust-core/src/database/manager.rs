use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use tokio::sync::mpsc;

use super::{
    client::DbClient,
    requests::{ReadRequest, WriteRequest},
    workers::{DbReader, DbWriter},
};
use crate::ffi_error::FfiError;

#[derive(Debug)]
pub enum DbMode {
    ReadOnly,
    ReadWrite,
}

pub fn start_conn(db_path: &Path, mode: DbMode) -> Result<Connection> {
    let flags = match mode {
        DbMode::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
        DbMode::ReadWrite => {
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
        }
    };

    let conn = Connection::open_with_flags(db_path, flags).map_err(|e| {
        log::error!("[DB] open failed: {}", e);
        e
    })?;

    if matches!(mode, DbMode::ReadWrite) {
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");
    }

    Ok(conn)
}

#[derive(uniffi::Object)]
pub struct DbManager {
    db_path: PathBuf,
    reader_tx: mpsc::Sender<ReadRequest>,
    writer_tx: mpsc::Sender<WriteRequest>,
    reader_handle: Option<JoinHandle<()>>,
    writer_handle: Option<JoinHandle<()>>,
}

impl DbManager {
    fn initialize_db(db_path: &Path) -> Result<()> {
        log::info!("[DB-MANAGER] initializing database");

        let conn = start_conn(db_path, DbMode::ReadWrite)?;
        conn.execute_batch(include_str!("sql/schema.sql"))?;

        log::info!("[DB-MANAGER] database initialized");

        Ok(())
    }

    fn spawn_workers(
        db_path: &PathBuf,
    ) -> (
        mpsc::Sender<ReadRequest>,
        mpsc::Sender<WriteRequest>,
        JoinHandle<()>,
        JoinHandle<()>,
    ) {
        let (reader_tx, reader_rx) = mpsc::channel::<ReadRequest>(32);
        let (writer_tx, writer_rx) = mpsc::channel::<WriteRequest>(32);

        let writer_db_path = db_path.clone();
        let writer_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let writer = DbWriter::start(writer_rx, writer_db_path)?;
                writer.request_loop();
                Ok(())
            })();

            if let Err(error) = result {
                log::error!("[DB-WRITER] exited with error: {}", error);
            } else {
                log::warn!("[DB-WRITER] exited without error");
            }
        });

        let reader_db_path = db_path.clone();
        let reader_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let reader = DbReader::start(reader_rx, reader_db_path)?;
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

    fn new(db_path: PathBuf) -> Result<Self> {
        Self::initialize_db(&db_path)?;

        let (reader_tx, writer_tx, reader_handle, writer_handle) = Self::spawn_workers(&db_path);

        Ok(Self {
            db_path,
            reader_tx,
            writer_tx,
            reader_handle: Some(reader_handle),
            writer_handle: Some(writer_handle),
        })
    }
}

#[uniffi::export]
impl DbManager {
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Self, FfiError> {
        (move || -> anyhow::Result<Self> {
            let db_path = PathBuf::from_str(&db_path_string)?;

            log::info!("[DB-MANAGER] spawning workers");

            Self::new(db_path)
        })()
        .map_err(FfiError::from)
    }

    //usefull for schema changes before we implement a proper migration plan
    #[uniffi::constructor]
    pub fn reset(db_path_string: String) -> Result<Self, FfiError> {
        (move || -> anyhow::Result<Self> {
            let db_path = PathBuf::from_str(&db_path_string)?;

            if db_path.exists() {
                std::fs::remove_file(&db_path)?;
            }

            log::info!("[DB-MANAGER] spawning fresh database");

            Self::new(db_path)
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