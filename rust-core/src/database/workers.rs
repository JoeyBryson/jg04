use std::path::{self, PathBuf};
use tokio::sync::mpsc;
use rusqlite::{Connection, OpenFlags};
use anyhow::Result;

mod reads;
mod writes;
use super::requests::{ReadRequest, WriteRequest};

pub struct DbReader{
    rx: mpsc::Receiver<ReadRequest>,
    conn: rusqlite::Connection
}

pub struct DbWriter{
    rx: mpsc::Receiver<WriteRequest>,
    conn: rusqlite::Connection
}


#[derive(Debug)]
pub enum DbMode {
    ReadOnly,
    ReadWrite,
}

fn start_conn(db_path: &path::Path, mode: DbMode) -> Result<Connection> {
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

impl DbReader {
    pub fn start(worker_rx: mpsc::Receiver<ReadRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[DB-READER] start db_path={:?}", db_path);
        let conn = start_conn(&db_path, DbMode::ReadOnly)?;
        log::info!("[DB-READER] started");
        Ok(Self { rx: worker_rx, conn })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-READER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            self.dispatch(request);
        }

        log::info!("[DB-READER] loop exit");
    }
}

impl DbWriter {
    pub fn start(worker_rx: mpsc::Receiver<WriteRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[DB-WRITER] start db_path={:?}", db_path);
        let conn = start_conn(&db_path, DbMode::ReadWrite)?;
        log::info!("[DB-WRITER] ready");
        Ok(Self { rx: worker_rx, conn })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-WRITER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            self.dispatch(request);
        }

        log::info!("[DB-WRITER] loop exit");
    }
}