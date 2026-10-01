//! Networking state and public API for the application.
//!
//! [`NwCore`] owns the long-lived networking components used by the application.
//! It provides the interface used by the UI to perform networking operations,
//! while delegating chat and invitation handling to dedicated managers.
//!
//! The networking runtime is owned by [`NwCore`] and runs independently of the
//! thread from which the object was created. Database access is performed
//! through [`DbClient`], which provides the persistent state required by the
//! networking layer.

use iroh::protocol::Router;
use iroh_gossip::net::Gossip;
use tokio::runtime::Handle;
use anyhow::Context;
use iroh::Endpoint;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use iroh::endpoint::presets;
use std::sync::Arc;
use iroh_gossip::proto::TopicId;
use tokio::runtime::Runtime;
use crate::ui::UiContact;
use super::{NwProfile, control_protocol::chat_invite::ChatInviteManager, groupchat::ChatSessionManager};
use super::{NwChat, NwContact, NwChatMember, NwChatMemberStatus};
use super::{CONTROL_ALPN, ControlProtocol};

/// Central state holder and public API for the networking layer.
///
/// `NwCore` owns the components responsible for communicating with peers,
/// managing chat sessions, and processing chat invitations. It also retains a copy of the 
/// database client used by those components to persist networking state.
///
/// The object owns a dedicated Tokio runtime so that networking tasks can
/// continue running independently of the thread that created `NwCore`.
///
/// The UI interacts with networking through the methods exposed by this type.
/// Operations that require ongoing asynchronous work are delegated to the
/// appropriate internal manager.
#[derive(uniffi::Object)]
pub struct NwCore {
    runtime_handle: Handle,
    chat_session_manager: ChatSessionManager,
    chat_invite_actor: ChatInviteManager,
}

#[uniffi::export]
impl NwCore {
    
    #[uniffi::constructor]
    pub fn spawn(db_client: Arc<DbClient>) -> Result<Self, FfiError> {
        let db_client = Arc::unwrap_or_clone(db_client);
        Self::spawn_with_preset(db_client, presets::N0)
            .map_err(FfiError::from)
    }

    pub fn send_message(&self, content: String, chat_id: String) -> Result<(), FfiError> {
        let mut topic_id_bytes = [0u8; 32];

        hex::decode_to_slice(&chat_id, &mut topic_id_bytes)
            .map_err(anyhow::Error::from)?;

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

impl NwCore {
    pub fn spawn_with_preset(
        db_client: DbClient,
        preset: impl presets::Preset,
    ) -> anyhow::Result<Self> {

        let runtime = Runtime::new().map_err(anyhow::Error::from)?;
        let runtime_handle = runtime.handle().clone();

        let nw_core = runtime_handle.block_on(Self::spawn_base(
            db_client.clone(),
            preset,
        ))?;

        Ok(nw_core)
    }

    async fn spawn_base(
        db_client: DbClient,
        preset: impl presets::Preset,
    ) -> anyhow::Result<Self> {
        let profile = db_client
            .get_nw_profile()
            .map_err(anyhow::Error::from)
            .with_context(|| "invalid private key")?;

        let endpoint = Endpoint::builder(presets::Minimal)
            .preset(preset)
            .secret_key(profile.secret_key.clone())
            .alpns(vec![
                CONTROL_ALPN.to_vec(),
                iroh_gossip::ALPN.to_vec(),
            ])
            .bind()
            .await
            .context("endpoint failed to bind")?;

        let gossip = Gossip::builder().spawn(endpoint.clone());

        let chat_session_manager = ChatSessionManager::spawn(
            db_client.clone(),
            gossip.clone(),
            profile.secret_key.clone(),
        ).await;

        let control_protocol = ControlProtocol {
            profile,
            db_client: db_client.clone(),
            chat_session_manager: chat_session_manager.clone(),
        };

        let router = Router::builder(endpoint)
            .accept(CONTROL_ALPN, control_protocol)
            .accept(iroh_gossip::ALPN, gossip)
            .spawn();

        let chat_invite_actor =
            ChatInviteManager::spawn(router, db_client.clone()).await?;

        Ok(Self {
            runtime_handle: Handle::current(),
            chat_session_manager,
            chat_invite_actor,
        })
    }
}