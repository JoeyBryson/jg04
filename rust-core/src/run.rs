use std::thread;
use rusqlite::Error::ToSqlConversionFailure;
use tokio::{runtime::Runtime, sync::{mpsc}};
use std::result::Result;
// use crate::nw::network_engine;
use crate::nw::NwDbManager;
use crate::ui::UiDbError;
use std::path::PathBuf;
use crate::notifications::{UiEventListener, UiEvent};





// fn main() -> Result<()>{
//     let (db_tx, db_rx) = mpsc::channel::<Command>(32); //Rust Engine to DataBase
//     let (db_panic_tx, db_panic_rx) = tokio::sync::oneshot::channel();

//     thread::spawn(move || {
//     let result = std::panic::catch_unwind(move || {
//             worker(db_rx, "app.db");
//         });

//         if let Err(err) = result {
//             let _ = db_panic_tx.send(err);
//         }
//     });

//     let ne_rt = Runtime::new()?;

//     ne_rt.block_on(async {
//         tokio::select! {
//             ne_result = network_engine(db_tx) => {
//                 ne_result
//             },
//             err = db_panic_rx => {
//                 Err(anyhow!("DB thread panicked: {err:?}"))
//             }
//         }
//     })

// }


fn parse_path(path_str: &str) -> PathBuf {
    // This is completely infallible and converts the &str directly into a PathBuf
    PathBuf::from(path_str)
}


#[uniffi::export]
pub fn delete_db(db_path_string: String) -> Result<(), UiDbError> {
    let db_path = parse_path(&db_path_string);
    if db_path.exists() {
        std::fs::remove_file(&db_path).map_err(|e| UiDbError::InternalError {
            msg: format!("Invalid path: {}", e),
        })?;
    };
    Ok(())
}
struct NoopListener;

impl UiEventListener for NoopListener {
    fn on_event(&self, _event: UiEvent) {}
}


#[uniffi::export]
pub fn add_sample_messages(db_path_string: String) -> Result<(), UiDbError>{

    let manager = NwDbManager::spawn(parse_path(&db_path_string))?;

    let client = manager.create_client(Box::new(NoopListener));

    let rt = Runtime::new().map_err(|e| UiDbError::InternalError { msg: e.to_string() })?;

    rt.block_on(async move {
        client.add_sample_chat().await;
    });

    Ok(())
}

