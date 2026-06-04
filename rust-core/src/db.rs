mod nw_requests;
mod ui_requests;
pub mod worker;
mod manager;
use std::thread::JoinHandle;
use std::path::PathBuf;
use tokio::sync::mpsc;
use anyhow::Result;
// use crate::nw::network_engine;

pub const SCHEMA: &str = include_str!("../sql/schema.sql");

// enum DbWorkerStatus {
//     Running,
//     Exited(anyhow::Result<()>)
// }

///DbWorker lives on a dedicated thread
pub struct DbWorker<TRequest>{
    worker_rx: mpsc::Receiver<TRequest>,
    conn: rusqlite::Connection
}

pub struct DbManager<TRequest> {
    worker_tx: mpsc::Sender<TRequest>,
    _join_handle: JoinHandle<()>,
}

pub trait WorkerImplemented: Sized {
    type Request;

    fn request_loop(self);

    fn start(
        worker_rx: mpsc::Receiver<Self::Request>,
        db_path: PathBuf,
    ) -> Result<Self>;
}
