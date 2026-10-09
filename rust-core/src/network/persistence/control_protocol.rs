use anyhow::Result;
use iroh::EndpointId;

use crate::database::client::DbClient;
use crate::network::NwContact;
use crate::network::NwProfile;

#[derive(Clone, Debug)]
pub(in crate::network) struct ControlProtocolStore {
    db: DbClient,
    profile: NwProfile
}

impl ControlProtocolStore {
    pub(in crate::network) async fn new(db: DbClient) -> Result<Self> {
        let profile = db.get_nw_profile_async().await?;
        Ok(Self { db, profile})
    }

    pub(in crate::network) async fn contact(&self, endpoint_id: EndpointId) -> Result<NwContact> {
        Ok(self.db.get_nw_contact(endpoint_id).await?)
    }

    pub(in crate::network) fn profile(&self) -> &NwProfile {
        &self.profile
    }
}