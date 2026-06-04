use super::{NwDbManager, NwDbClient};
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
