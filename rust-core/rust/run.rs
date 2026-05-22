use std::thread;
use tokio::{runtime::Runtime, sync::{mpsc}};
use anyhow::{Result, anyhow};
use crate::{db::{Command, worker}, nw};
use crate::nw::network_engine;
use crate::ui;
use std::any::Any;
use std::sync::{Arc, Mutex};
use crate::logging::{LogLevel};





fn main() -> Result<()>{
    let (db_tx, db_rx) = mpsc::channel::<Command>(32); //Rust Engine to DataBase
    let (db_panic_tx, db_panic_rx) = tokio::sync::oneshot::channel();

    thread::spawn(move || {
    let result = std::panic::catch_unwind(move || {
            worker(db_rx, "app.db");
        });

        if let Err(err) = result {
            let _ = db_panic_tx.send(err);
        }
    });

    let ne_rt = Runtime::new()?;

    ne_rt.block_on(async {
        tokio::select! {
            ne_result = network_engine(db_tx) => {
                ne_result
            },
            err = db_panic_rx => {
                Err(anyhow!("DB thread panicked: {err:?}"))
            }
        }
    })

}

#[uniffi::export]
pub fn setup_test_db(db_path: String) -> ui::DbEntrypoint {
    log::info!("yeahhhhh");
    log::error!("TEST INFO LOGGING-- IT'S ALSO WORKING");

    let worker_status: Arc<Mutex<Option<Result<()>>>> = Arc::new(Mutex::new(None));
    let worker_status_clone = worker_status.clone();

    let (db_tx, db_rx) = mpsc::channel::<Command>(32);

    let join_handle = thread::spawn(move || {
        let result = worker(db_rx, db_path);
        *worker_status_clone.lock().unwrap() = Some(result);
    });

    let ne_rt = Runtime::new().expect("RN001: runtime init");
    let nw_entrypoint = nw::create_async_db_entrypoint(db_tx.clone());

    ne_rt.block_on(async {
        let _ = nw_entrypoint.add_sample_chat().await;
    });

    let ui_entrypoint = ui::create_db_entrypoint(db_tx, join_handle, worker_status);
    ui_entrypoint
}


