use std::path::PathBuf;
use tokio::sync::mpsc;
use anyhow::Result;

use crate::ffi_error::FfiError;
use std::str::FromStr;
use super::{DbManager, DbClient, DbReader, DbWriter, ReadRequest, WriteRequest};
use std::sync::Arc;

#[uniffi::export]
impl DbManager {
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Self, FfiError> {
        (move || -> anyhow::Result<Self> {
            let db_path = PathBuf::from_str(&db_path_string)?;

            log::info!("[DB-MANAGER] spawning workers");

            let (reader_tx, reader_rx) = mpsc::channel::<ReadRequest>(32);
            let (writer_tx, writer_rx) = mpsc::channel::<WriteRequest>(32);

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

            let writer_handle = std::thread::spawn(move || {
                let result = (|| -> Result<()> {
                    let writer = DbWriter::start(writer_rx, db_path)?;
                    writer.request_loop();
                    Ok(())
                })();

                if let Err(error) = result {
                    log::error!("[DB-WRITER] exited with error: {}", error);
                } else {
                    log::warn!("[DB-WRITER] exited without error");
                }
            });

            Ok(Self {
                reader_tx,
                writer_tx,
                _reader_handle: reader_handle,
                _writer_handle: writer_handle,
            })
        })()
        .map_err(FfiError::from)
    }

    pub fn spawn_client(&self) -> Arc<DbClient> {
        Arc::new(DbClient {
            reader_tx: self.reader_tx.clone(),
            writer_tx: self.writer_tx.clone(),
        })
    }
}