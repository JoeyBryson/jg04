use std::time::SystemTime;

use anyhow::Result;
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;

use crate::database::client::DbClient;
use crate::network::chat::ChatSessionsHandle;
use crate::network::control::{ChatInvitesHandle, ControlProtocol};
use crate::network::messaging::signed::MessageData;
use crate::network::persistence::{InviteStore, SessionManagerStore};
use crate::network::{NwChatMember, CONTROL_ALPN};

/// Coordinates networking use cases across persisted state, live chat
/// sessions, and invitation delivery.
pub(in crate::network) struct NwService {
    chat_sessions: ChatSessionsHandle,
    chat_invites: ChatInvitesHandle,
}

impl NwService {
    pub(in crate::network) async fn spawn(
        db_client: DbClient,
        preset: impl presets::Preset,
    ) -> Result<Self> {
        let profile = db_client.get_nw_profile_async().await?;

        let endpoint = Endpoint::builder(preset)
            .secret_key(profile.secret_key.clone())
            .alpns(vec![CONTROL_ALPN.to_vec(), iroh_gossip::ALPN.to_vec()])
            .bind()
            .await?;

        let gossip = Gossip::builder().spawn(endpoint.clone());
        let chat_sessions = ChatSessionsHandle::spawn(
            gossip.clone(),
            SessionManagerStore::new(db_client.clone()).await?,
        )
        .await?;

        let control_protocol = ControlProtocol {
            chat_sessions: chat_sessions.clone(),
        };

        let router = Router::builder(endpoint)
            .accept(CONTROL_ALPN, control_protocol)
            .accept(iroh_gossip::ALPN, gossip)
            .spawn();

        let chat_invites =
            ChatInvitesHandle::spawn(router, InviteStore::new(db_client.clone())).await?;

        Ok(Self {
            chat_sessions,
            chat_invites,
        })
    }

    pub(in crate::network) async fn create_chat(
        &self,
        members: Vec<NwChatMember>,
        name: Option<String>,
    ) -> Result<String> {
        log::info!(
            "[NW-SERVICE] create_chat requested: members={}, named={}",
            members.len(),
            name.is_some()
        );

        let topic_id = self
            .chat_sessions
            .request_create_chat(members, name)
            .await?;

        log::info!("[NW-SERVICE] create_chat started session for topic={topic_id}");

        self.chat_invites.request_invite_delivery().await?;

        log::debug!("[NW-SERVICE] invite delivery requested for topic={topic_id}");

        Ok(topic_id.to_string())
    }

    pub(in crate::network) async fn send_message(
        &self,
        topic_id: TopicId,
        content: String,
    ) -> Result<()> {
        let sent_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message_data = MessageData { content, sent_at };

        log::debug!(
            "[NW-SERVICE] send_message: topic={}, sent_at={}, content_len={}",
            topic_id,
            message_data.sent_at,
            message_data.content.len()
        );

        self.chat_sessions
            .request_send_message(topic_id, message_data)
            .await?;

        log::debug!("[NW-SERVICE] send_message queued to chat session: topic={topic_id}");

        Ok(())
    }

    pub(in crate::network) async fn shutdown(&self) -> Result<()> {
        self.chat_invites.shutdown().await
    }
}
