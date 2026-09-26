//!

use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;
use tokio::sync::mpsc;

mod nw_reads;
mod ui_reads;
mod writes;

#[cfg(test)]
mod test;

use super::manager::{DbMode, start_conn};
use super::client::{ReadRequest, WriteRequest};

pub struct DbReader {
    rx: mpsc::Receiver<ReadRequest>,
    conn: Connection,
}

pub struct DbWriter {
    rx: mpsc::Receiver<WriteRequest>,
    conn: Connection,
}

impl DbReader {
    pub fn start(worker_rx: mpsc::Receiver<ReadRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[DB-READER] start db_path={:?}", db_path);

        let conn = start_conn(&db_path, DbMode::ReadOnly)?;

        log::info!("[DB-READER] started");

        Ok(Self {
            rx: worker_rx,
            conn,
        })
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

        Ok(Self {
            rx: worker_rx,
            conn,
        })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-WRITER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            self.dispatch(request);
        }

        log::info!("[DB-WRITER] loop exit");
    }
}
