use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use iroh::endpoint::{presets, Connection};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::Endpoint;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Handle, Runtime};
use super::NwCore;
use super::super::{groupchat::ChatSession, CONTROL_ALPN, 
    NwContact, NwProfile, ControlProtocol, ControlMessage};

use super::NwChat;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{};
use crate::ui::UiContact;
use crate::ui::UiChatHeader;


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

        let mut nw_core = runtime_handle.block_on(
            Self::spawn_base(db_client, profile)
        )?;

        let chats = nw_core.db_client.get_nw_chats_sync()?;

        for chat in chats {
            nw_core.spawn_chat_connector(chat)?;
        }

        let nw_core = Arc::new(nw_core);

        //To-do: is this necessary?
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

        let connector = self
            .chat_connectors
            .get(&topic_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No active connector for topic ID: {}",
                    chat_id
                )
            })?;

        self.runtime_handle
            .block_on(connector.send(content))
            .map_err(anyhow::Error::from)?;

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
            if let Err(error) = Self::invite_chat_members(router, chat_clone).await {
                log::error!("failed to invite chat members: {error}");
            }
        });

        self.db_client
            .add_nw_chat_sync(chat.clone())?;

        self.spawn_chat_connector(chat)?;

        Ok(topic_id.to_string())
    }
}