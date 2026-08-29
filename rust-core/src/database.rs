mod requests;
pub mod workers;
mod manager;
use std::thread::JoinHandle;
use tokio::sync::mpsc;

use crate::ui::UiDbRequest;
use crate::network::NwDbRequest;

pub struct UiDbWorker{
    worker_rx: mpsc::Receiver<UiDbRequest>,
    conn: rusqlite::Connection
}

pub struct NwDbWorker{
    worker_rx: mpsc::Receiver<NwDbRequest>,
    conn: rusqlite::Connection
}

#[derive(uniffi::Object)]
pub struct UiDbManager {
    worker_tx: mpsc::Sender<UiDbRequest>,
    _join_handle: JoinHandle<()>,
}

pub struct NwDbManager {
    worker_tx: mpsc::Sender<NwDbRequest>,
    _join_handle: JoinHandle<()>,
}

