//! Database workers which run in their own threads and own their own database connections.
//! The purpose of this design is to keep database connections alive somewhere that can be accessed from anywhere in the app.
//! Channels provide a natural way to queue database work on a single connection.
//! Using the tokio variant  [`tokio::sync::mpsc`] of the mpsc primitive allows for natural integration for async database calls, allowing the app to do other work
//! while the data is being fetched or written.
//! Sync database calls are also possible via the .blocking_send and .blocking_recv methods.
//! The [`DbReader`] and [`DbWriter`] workers each live on their own threads. This is to keep writes from blocking reads, 
//! As WAL mode on SQLight allows for many readers - one read/writer concurrently. Writes can take orders of magnitude longer
//! and UI responsiveness is a priority. This may be over engineering - but it's done now.

use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;
use tokio::sync::mpsc;

mod nw_reads;
mod ui_reads;
mod writes;

#[cfg(test)]
mod test;

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
    pub fn new(worker_rx: mpsc::Receiver<ReadRequest>, conn: Connection) -> Result<Self> {
        Ok(Self {
            rx: worker_rx,
            conn,
        })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-READER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            //dispatch is generated in super::client.rs using macros from super::macros.rs
            //it is effectively just a match statement that matches the request enum to internal methods
            self.dispatch(request);
        }

        log::info!("[DB-READER] loop exit");
    }
}

impl DbWriter {
    pub fn new(worker_rx: mpsc::Receiver<WriteRequest>, conn: Connection) -> Result<Self> {
        Ok(Self {
            rx: worker_rx,
            conn,
        })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-WRITER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            //^^
            self.dispatch(request);
        }

        log::info!("[DB-WRITER] loop exit");
    }
}
