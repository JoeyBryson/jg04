use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh_gossip::proto::TopicId;
use tokio::runtime::Runtime;

use super::NwCore;
use super::super::{
    ChatSessionManager,
    ControlMessage,
    ControlProtocol,
    NwChat,
    NwContact,
    NwProfile,
    CONTROL_ALPN,
};

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

        let runtime = Runtime::new()
            .map_err(anyhow::Error::from)?;

        let runtime_handle = runtime.handle().clone();

        let nw_core = runtime_handle.block_on(
            Self::spawn_base(db_client, profile)
        )?;

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

    fn send_message(
        &self,
        content: String,
        chat_id: String,
    ) -> Result<(), FfiError> {
        let mut topic_id_bytes = [0u8; 32];

        hex::decode_to_slice(&chat_id, &mut topic_id_bytes)
            .map_err(anyhow::Error::from)?;

        let topic_id = TopicId::from_bytes(topic_id_bytes);

        self.chat_session_manager
            .send_message(topic_id, content)?;

        Ok(())
    }

    fn crate_chat(
        &self,
        contacts: Vec<UiContact>,
        name: Option<String>,
    ) -> Result<String, FfiError> {
        let members = contacts
            .into_iter()
            .map(NwContact::from)
            .collect::<Vec<NwContact>>();

        let topic_id = TopicId::from_bytes(rand::random());

        let chat = NwChat {
            name,
            members,
            topic_id,
        };

        let chat_clone = chat.clone();
        let router = self.router.clone();

        self.runtime_handle.spawn(async move {
            if let Err(error) =
                Self::invite_chat_members(router, chat_clone).await
            {
                log::error!("failed to invite chat members: {error}");
            }
        });

        self.db_client
            .add_nw_chat_sync(chat.clone())?;

        self.chat_session_manager.add_chat(chat)?;

        Ok(topic_id.to_string())
    }
}