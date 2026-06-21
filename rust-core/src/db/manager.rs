use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use anyhow::Result;

use crate::nw::NwDbRequest;
use crate::ui::UiDbRequest;

use super::{UiDbManager, UiDbWorker, NwDbManager, NwDbWorker};

// fn ensure_parent_dir(db_path: &Path) -> Result<()> {
//     if let Some(parent) = db_path.parent() {
//         std::fs::create_dir_all(parent)?;
//         log::info!("[DB-MANAGER] created directory path={:?}", parent);
//     }
//     Ok(())
// }


impl UiDbManager {
    pub fn spawn(db_path: PathBuf) -> Result<Self> {
        // ensure_parent_dir(&db_path)?;
        log::info!("[UI-MANAGER] spawning worker");

        let (worker_tx, worker_rx) = mpsc::channel::<UiDbRequest>(32);

        let join_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let worker = UiDbWorker::start(worker_rx, db_path)?;
                worker.request_loop();
                Ok(())
            })();

            if let Err(error) = result {
                log::error!("[UI-WORKER] exited with error: {}", error);
            } else {
                log::warn!("[UI-WORKER] exited without error");
            }
        });

        Ok(Self {
            _join_handle: join_handle,
            worker_tx,
        })
    }

    pub fn worker_tx(&self) -> mpsc::Sender<UiDbRequest> {
        self.worker_tx.clone()
    }
}

impl NwDbManager {
    pub fn spawn(db_path: PathBuf) -> Result<Self> {
        // ensure_parent_dir(&db_path)?;
        log::info!("[NW-MANAGER] spawning worker");

        let (worker_tx, worker_rx) = mpsc::channel::<NwDbRequest>(32);

        let join_handle = std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let worker = NwDbWorker::start(worker_rx, db_path)?;
                worker.request_loop();
                Ok(())
            })();

            if let Err(error) = result {
                log::error!("[NW-WORKER] exited with error: {}", error);
            } else {
                log::warn!("[NW-WORKER] exited without error");
            }
        });

        Ok(Self {
            _join_handle: join_handle,
            worker_tx,
        })
    }

    pub fn worker_tx(&self) -> mpsc::Sender<NwDbRequest> {
        self.worker_tx.clone()
    }
}