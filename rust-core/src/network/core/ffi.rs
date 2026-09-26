use std::sync::Arc;

use anyhow::Context;
use iroh_gossip::proto::TopicId;
use tokio::runtime::Runtime;

use super::super::{NwChat, NwChatMember, NwChatMemberStatus, NwContact};
use super::NwCore;

use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::ui::UiContact;

#[uniffi::export]
impl NwCore {
    #[uniffi::constructor]
    pub fn spawn(db_client: Arc<DbClient>) -> Result<Arc<Self>, FfiError> {
        log::info!("starting nw_core::spawn");

        let db_client = Arc::unwrap_or_clone(db_client);

        let profile = db_client
            .get_nw_profile()
            .map_err(anyhow::Error::from)
            .with_context(|| "invalid private key")?;

        let runtime = Runtime::new().map_err(anyhow::Error::from)?;

        let runtime_handle = runtime.handle().clone();

        let nw_core = runtime_handle.block_on(Self::spawn_base(db_client, profile))?;

        runtime_handle.block_on(nw_core.chat_invite_actor.refresh())?;

        let chats = nw_core.db_client.get_nw_chats_sync()?;

        for chat in chats {
            nw_core.chat_session_manager.add_chat(chat)?;
        }

        let nw_core = Arc::new(nw_core);

        std::thread::spawn(move || {
            runtime.block_on(std::future::pending::<()>());
        });

        log::info!("nw_core::spawn completed");

        Ok(nw_core)
    }

    pub fn send_message(&self, content: String, chat_id: String) -> Result<(), FfiError> {
        let mut topic_id_bytes = [0u8; 32];

        hex::decode_to_slice(&chat_id, &mut topic_id_bytes).map_err(anyhow::Error::from)?;

        let topic_id = TopicId::from_bytes(topic_id_bytes);

        self.chat_session_manager.send_message(topic_id, content)?;

        Ok(())
    }

    pub fn create_chat(
        &self,
        contacts: Vec<UiContact>,
        name: Option<String>,
    ) -> Result<String, FfiError> {
        let members = contacts
            .into_iter()
            .map(NwContact::from)
            .map(|contact| NwChatMember {
                contact,
                status: NwChatMemberStatus::Pending,
            })
            .collect::<Vec<NwChatMember>>();

        let topic_id = TopicId::from_bytes(rand::random());

        let chat = NwChat {
            name,
            members,
            topic_id,
        };

        self.db_client.add_nw_chat_sync(chat.clone())?;

        self.chat_session_manager.add_chat(chat)?;

        let chat_invite_actor = self.chat_invite_actor.clone();
        self.runtime_handle.spawn(async move {
            if let Err(error) = chat_invite_actor.refresh().await {
                log::error!("failed to refresh chat invites: {error}");
            }
        });

        Ok(topic_id.to_string())
    }
}

/// Test-only constructor used by [`crate::harness`] to bind against a local
/// relay/DNS pair instead of the production `N0` defaults.
#[cfg(feature = "test-utils")]
impl NwCore {
    pub fn spawn_for_test(
        db_client: Arc<DbClient>,
        preset: impl iroh::endpoint::presets::Preset,
    ) -> Result<Arc<Self>, FfiError> {
        let db_client = Arc::unwrap_or_clone(db_client);

        let profile = db_client
            .get_nw_profile()
            .map_err(anyhow::Error::from)
            .with_context(|| "invalid private key")?;

        let runtime = Runtime::new().map_err(anyhow::Error::from)?;

        let runtime_handle = runtime.handle().clone();

        let nw_core =
            runtime_handle.block_on(Self::spawn_base_with_preset(db_client, profile, preset))?;

        runtime_handle.block_on(nw_core.chat_invite_actor.refresh())?;

        let chats = nw_core.db_client.get_nw_chats_sync()?;

        for chat in chats {
            nw_core.chat_session_manager.add_chat(chat)?;
        }

        let nw_core = Arc::new(nw_core);

        std::thread::spawn(move || {
            runtime.block_on(std::future::pending::<()>());
        });

        Ok(nw_core)
    }
}
