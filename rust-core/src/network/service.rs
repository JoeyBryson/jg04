use std::time::SystemTime;

use anyhow::Result;
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;

use crate::database::client::DbClient;
use crate::network::control_protocol::chat_invite::ChatInviteManager;
use crate::network::signed_message::MessageData;
use crate::network::stores::{InviteStore, SessionManagerStore};
use crate::network::{NwChatMember, NwProfile};
use crate::network::{groupchat::ChatSessionManager, CONTROL_ALPN, ControlProtocol};

/// Coordinates networking use cases across persisted state, live chat
/// sessions, and invitation delivery.
pub(super) struct NwService {
    chat_session_manager: ChatSessionManager,
    chat_invite_manager: ChatInviteManager,
}

impl NwService {
    pub(super) async fn spawn(
        db_client: DbClient,
        profile: NwProfile,
        preset: impl presets::Preset,
    ) -> Result<Self> {
        let endpoint = Endpoint::builder(preset)
            .secret_key(profile.secret_key.clone())
            .alpns(vec![CONTROL_ALPN.to_vec(), iroh_gossip::ALPN.to_vec()])
            .bind()
            .await?;

        let gossip = Gossip::builder().spawn(endpoint.clone());
        let chat_session_manager = ChatSessionManager::spawn(
            gossip.clone(),
            profile.clone(),
            SessionManagerStore::new(db_client.clone()),
        )
        .await?;

        let control_protocol = ControlProtocol {
            profile: profile.clone(),
            db_client: db_client.clone(),
            chat_session_manager: chat_session_manager.clone(),
        };

        let router = Router::builder(endpoint)
            .accept(CONTROL_ALPN, control_protocol)
            .accept(iroh_gossip::ALPN, gossip)
            .spawn();

        let chat_invite_manager = ChatInviteManager::spawn(router, InviteStore::new(db_client.clone())).await?;

        Ok(Self {
            chat_session_manager,
            chat_invite_manager,
        })
    }

    pub(super) async fn create_chat(
        &self,
        members: Vec<NwChatMember>,
        name: Option<String>,
    ) -> Result<String> {
        let topic_id = self.chat_session_manager.create_chat(members, name).await?;

        self.chat_invite_manager.refresh().await?;

        Ok(topic_id.to_string())
    }

    pub(super) async fn send_message(&self, topic_id: TopicId, content: String) -> Result<()> {
        let sent_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message_data = MessageData { content, sent_at };

        self.chat_session_manager
            .send_message(topic_id, message_data)
            .await?;

        Ok(())
    }
}
