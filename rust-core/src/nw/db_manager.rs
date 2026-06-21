use super::{NwDbManager, NwDbClient};


impl NwDbManager {
    pub fn create_client(
        &self
    ) -> NwDbClient {

        let worker_tx = self.worker_tx();

        NwDbClient {
            worker_tx
        }
    }
}
