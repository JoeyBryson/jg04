use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use anyhow::Result;

use crate::ffi_error::FfiError;
use std::{str::FromStr};
use super::{UiDbManager, UiDbWorker, NwDbManager, NwDbWorker, UiDbRequest, UiDbClient, NwDbClient, NwDbRequest};
use std::sync::Arc;


#[uniffi::export]
impl UiDbManager {
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Self, FfiError> {
        (move || -> anyhow::Result<Self> {
            let db_path = PathBuf::from_str(&db_path_string)?;

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
        })()
        .map_err(FfiError::from)
    }

    pub fn spawn_client(&self) -> Arc<UiDbClient> {
        Arc::new(UiDbClient {
            worker_tx: self.worker_tx.clone(),
        })
    }
}

#[uniffi::export]
impl NwDbManager {
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Self, FfiError> {
        (move || -> anyhow::Result<Self> {
            let db_path = PathBuf::from_str(&db_path_string)?;

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
        })()
        .map_err(FfiError::from)
    }

    pub fn spawn_client(&self) -> Arc<NwDbClient> {
        Arc::new(NwDbClient {
            worker_tx: self.worker_tx.clone(),
        })
    }
}