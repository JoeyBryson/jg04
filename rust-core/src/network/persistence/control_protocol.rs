use anyhow::Result;
use iroh::EndpointId;

use crate::database::client::DbClient;
use crate::network::NwContact;

#[derive(Clone, Debug)]
pub(in crate::network) struct ControlProtocolStore {
    db: DbClient,
}

impl ControlProtocolStore {
    pub(in crate::network) fn new(db: DbClient) -> Self {
        Self { db }
    }

    pub(in crate::network) async fn contact(&self, endpoint_id: EndpointId) -> Result<NwContact> {
        Ok(self.db.get_nw_contact(endpoint_id).await?)
    }
}