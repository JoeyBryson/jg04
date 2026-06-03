use std::sync::{Arc, OnceLock};
use std::thread::{JoinHandle, spawn};
use std::path::{self, PathBuf};
use std::thread;
use std::sync::Mutex;
use tokio::{runtime::Runtime, sync::{mpsc}};
use anyhow::{Result};

use crate::db::{ NwDbClient};
use crate::db::{worker};

use super::{DbManager, DbWorker, WorkerImplemented};

// struct DbWorker<DbRequest>{
//     pub worker_rx: mpsc::Receiver<DbRequest>,
//     conn: rusqlite::Connection,
// }

// #[derive(Clone)]
// pub struct DbClient<DbRequest> {
//     pub worker_tx: mpsc::Sender<DbRequest>,
//     pub worker_status: Arc<Mutex<DbWorkerStatus>>,
// }

// pub struct DbManager<DbRequest> {
//     worker_tx: mpsc::Sender<DbRequest>,
//     worker_status: Arc<Mutex<DbWorkerStatus>>,
//     join_handle: JoinHandle<()>,
// }

impl<TRequest> DbManager<TRequest> 
where
    TRequest: Send + 'static,
    DbWorker<TRequest>: WorkerImplemented<Request = TRequest>, {
    pub fn delete_db_then_spawn(db_path: PathBuf) -> Result<Self> {
        if db_path.exists() {
            std::fs::remove_file(&db_path)?;
        };
        Self::spawn(db_path)
    }

    pub fn spawn(db_path: PathBuf) -> Result<Self> {

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let (worker_tx, worker_rx) = mpsc::channel::<TRequest>(32);

        let join_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let worker = DbWorker::start(worker_rx, db_path)?;
                worker.execute_schema()?;
                Ok(worker.request_loop())
            })();

            match result {
                Ok(()) => log::warn!("worker exited without error"),
                Err(error) => log::error!("worker exited with error: {}", error),
            }
        });

        Ok(Self {
            join_handle,
            worker_tx,
        })
    }

    pub fn worker_tx(&self) -> mpsc::Sender<TRequest> {
        self.worker_tx.clone()
    }
}