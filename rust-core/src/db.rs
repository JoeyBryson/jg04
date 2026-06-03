mod nw_requests;
mod ui_requests;
pub mod worker;
mod manager;
use std::sync::{Arc, OnceLock};
use std::thread::JoinHandle;
use std::path::{self, PathBuf};
use std::thread;
use std::sync::Mutex;
use tokio::{runtime::Runtime, sync::{mpsc}};
use anyhow::Result;
use crate::{db, nw};
// use crate::nw::network_engine;
use crate::ui;
use crate::nw::{NwDbClient, NwDbRequest};

const SCHEMA: &str = include_str!("../sql/schema.sql");

// enum DbWorkerStatus {
//     Running,
//     Exited(anyhow::Result<()>)
// }

///DbWorker lives on a dedicated thread
struct DbWorker<TRequest>{
    worker_rx: mpsc::Receiver<TRequest>,
    conn: rusqlite::Connection,
    db_path: PathBuf
}

pub struct DbManager<TRequest> {
    worker_tx: mpsc::Sender<TRequest>,
    join_handle: JoinHandle<()>,
}

pub trait WorkerImplemented: Sized {
    type Request;

    fn request_loop(self);

    fn start(
        worker_rx: mpsc::Receiver<Self::Request>,
        db_path: PathBuf,
    ) -> Result<Self>;
}

// impl <TRequest> DbWorker<TRequest> {
//     fn new<P: AsRef<path::Path>>(
//         db_rx: mpsc::Receiver<TRequest>,
//         path: P
//     ) -> Result<Self> {
//         todo!()
//     }
// }

// struct NwDbWorker {
//     conn: Connection,
// }

// struct UiDbWorker {
//     conn: Connection,
// }






// impl<TRequest> DbClient<TRequest> {
//     pub fn new(
//         db_tx: mpsc::Sender<TRequest>,
//         join_handle: JoinHandle<()>,
//         worker_status: Arc<OnceLock<anyhow::Result<()>>>,
//     ) -> Self {
//         Self {
//             db_tx,
//             join_handle,
//             worker_status,
//         }
//     }
// }

// impl <TRequest> DbClient<TRequest> {
//     pub fn new<P: AsRef<path::Path>>(db_path: P) -> Self {

//         let worker_status: Arc<OnceLock<anyhow::Result<()>>> =
//             Arc::new(OnceLock::new());

//         let worker_status_clone = worker_status.clone();

//         let (db_tx, db_rx) = mpsc::channel::<TRequest>(32);

//         let nw_join_handle = std::thread::spawn(move || {
//             let result = delete_db_then_start_nw_worker(db_rx, db_path);
//             worker_status_clone.set(result);
//         });


//         Self {
//             db_tx,
//             join_handle: nw_join_handle,
//             worker_status,
//         }
//     }
// }