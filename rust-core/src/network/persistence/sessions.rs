use anyhow::Result;
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;

use crate::database::client::DbClient;
use crate::network::{NwChat, NwMessage, NwProfile};

#[derive(Clone)]
pub(in crate::network) struct SessionManagerStore {
    db: DbClient,
    profile: NwProfile,
}

impl SessionManagerStore {
    pub(in crate::network) async fn new(db: DbClient) -> Result<Self> {
        let profile = db.get_nw_profile_async().await?;
        Ok(Self { db, profile })
    }

    pub(in crate::network) fn spawn_session_store(&self, chat: NwChat) -> SessionStore {
        SessionStore {
            db: self.db.clone(),
            chat,
            profile: self.profile.clone(),
        }
    }

    pub(in crate::network) async fn chats(&self) -> Result<Vec<NwChat>> {
        Ok(self.db.get_nw_chats().await?)
    }

    pub(in crate::network) async fn save_chat(&self, chat: NwChat) -> Result<()> {
        self.db.add_nw_chat(chat).await?;
        Ok(())
    }

    pub(in crate::network) fn profile(&self) -> &NwProfile {
        &self.profile
    }
}

pub(in crate::network) struct SessionStore {
    db: DbClient,
    chat: NwChat,
    profile: NwProfile,
}

impl SessionStore {
    pub(in crate::network) fn topic_id(&self) -> TopicId {
        self.chat.topic_id
    }

    pub(in crate::network) fn chat(&self) -> &NwChat {
        &self.chat
    }

    pub(in crate::network) fn profile(&self) -> &NwProfile {
        &self.profile
    }

    pub(in crate::network) async fn record_message(
        &self,
        sender: EndpointId,
        content: String,
        sent_at: i64,
    ) -> Result<()> {
        let topic_id = self.chat.topic_id;
        let content_len = content.len();

        log::debug!(
            "[SESSION-STORE] record_message start: topic={}, sender={}, sent_at={}, content_len={}",
            topic_id,
            sender,
            sent_at,
            content_len
        );

        self.db
            .add_nw_message(NwMessage {
                topic_id,
                endpoint_id: sender,
                content,
                sent_at,
            })
            .await?;

        log::debug!(
            "[SESSION-STORE] record_message done: topic={}, sender={}, sent_at={}",
            topic_id,
            sender,
            sent_at
        );

        Ok(())
    }
}