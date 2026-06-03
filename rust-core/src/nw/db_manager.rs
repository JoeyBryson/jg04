use std::{path::PathBuf, str::FromStr};
use super::{NwDbManager, NwDbRequest, NwDbClient};
use crate::db::{DbManager};
use crate::notifications::UiEventListener;
use std::sync::Arc;


impl NwDbManager {
    pub fn create_client(
        &self,
        listener: Box<dyn UiEventListener>,
    ) -> NwDbClient {

        let worker_tx = self.worker_tx();

        NwDbClient {
            worker_tx,
            ui_listener: Arc::from(listener),
        }
    }
}
