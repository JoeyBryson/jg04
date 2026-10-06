//! Narrow persistence handles so each network component can only touch the state it owns.

use anyhow::Result;
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;

use crate::database::client::DbClient;
use crate::network::{NwChat, NwMessage};

/// Hands out per-topic stores so a session can never write to another chat.
#[derive(Clone)]
pub(super) struct SessionManagerStore {
    db: DbClient,
}

impl SessionManagerStore {
    pub(super) fn new(db: DbClient) -> Self {
        Self { db }
    }

    pub(super) fn spawn_session_store(&self, topic_id: TopicId) -> SessionStore {
        SessionStore {
            db: self.db.clone(),
            topic_id,
        }
    }

    pub(super) async fn chats(&self) -> Result<Vec<NwChat>> {
        Ok(self.db.get_nw_chats().await?)
    }

    pub(super) async fn save_chat(&self, chat: NwChat) -> Result<()> {
        self.db.add_nw_chat(chat).await?;

        Ok(())
    }
}

#[derive(Clone)]
pub(super) struct SessionStore {
    db: DbClient,
    topic_id: TopicId,
}

impl SessionStore {
    pub(super) async fn record_message(
        &self,
        sender: EndpointId,
        content: String,
        sent_at: i64,
    ) -> Result<()> {
        self.db
            .add_nw_message(NwMessage {
                topic_id: self.topic_id,
                endpoint_id: sender,
                content,
                sent_at,
            })
            .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub(super) struct InviteStore {
    db: DbClient,
}

impl InviteStore {
    pub(super) fn new(db: DbClient) -> Self {
        Self { db }
    }

    pub(super) async fn chats(&self) -> Result<Vec<NwChat>> {
        Ok(self.db.get_nw_chats().await?)
    }

    pub(super) async fn mark_joined(&self, topic_id: TopicId, member: EndpointId) -> Result<()> {
        self.db
            .mark_nw_chat_member_joined(topic_id, member)
            .await?;

        Ok(())
    }
}
