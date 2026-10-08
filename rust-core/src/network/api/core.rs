use anyhow::Result;
use iroh::endpoint::presets;
use iroh_gossip::proto::TopicId;
use tokio::runtime::Runtime;

use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::NwChatMember;

use crate::network::application::NwService;

/// Synchronous facade and runtime owner for the networking layer. Async
/// application work is run on its retained runtime through [`NwService`].
pub struct NwCore {
    // Retained to keep networking tasks spawned on this runtime alive.
    runtime: Runtime,
    service: NwService,
}

impl NwCore {
    pub fn spawn(db_client: DbClient, preset: impl presets::Preset) -> Result<Self> {
        let profile = db_client.get_nw_profile()?;
        let runtime = Runtime::new()?;
        let service = runtime.block_on(NwService::spawn(db_client, profile, preset))?;

        Ok(Self { runtime, service })
    }

    pub fn send_message(&self, content: String, topic_id: TopicId) -> Result<(), FfiError> {
        self.runtime
            .block_on(self.service.send_message(topic_id, content))
            .map_err(FfiError::from)
    }

    pub fn create_chat(
        &self,
        members: Vec<NwChatMember>,
        name: Option<String>,
    ) -> Result<String, FfiError> {
        self.runtime
            .block_on(self.service.create_chat(members, name))
            .map_err(FfiError::from)
    }
}
