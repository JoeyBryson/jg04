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
use crate::network::persistence::{ControlProtocolStore, InviteStore, SessionManagerStore};
use crate::network::{NwChatMember, NwProfile, CONTROL_ALPN};

/// Coordinates networking use cases across persisted state, live chat
/// sessions, and invitation delivery.
pub(in crate::network) struct NwService {
    chat_sessions: ChatSessionsHandle,
    chat_invites: ChatInvitesHandle,
}

impl NwService {
    pub(in crate::network) async fn spawn(
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
        let chat_sessions = ChatSessionsHandle::spawn(
            gossip.clone(),
            SessionManagerStore::new(db_client.clone())?,
        )
        .await?;

        let control_protocol = ControlProtocol {
            profile: profile.clone(),
            store: ControlProtocolStore::new(db_client.clone()),
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
        let topic_id = self
            .chat_sessions
            .request_create_chat(members, name)
            .await?;

        self.chat_invites.request_invite_delivery().await?;

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

        self.chat_sessions
            .request_send_message(topic_id, message_data)
            .await?;

        Ok(())
    }
}
