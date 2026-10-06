use super::super::{
    NwChat,
    signed_message::{sign_and_encode, verify_and_decode},
};
use crate::network::stores::SessionStore;
use crate::network::{NwChatMember, NwProfile, profile};
use crate::network::signed_message::MessageData;
use futures_lite::StreamExt;
use iroh::SecretKey;
use iroh_gossip::api::Event as GossipEvent;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender},
    net::Gossip,
    proto::TopicId,
};
use std::time::SystemTime;
use tokio::task::JoinHandle;
use crate::network::NwChatMemberStatus;
pub struct ChatSession {
    pub profile: NwProfile,
    pub topic_id: TopicId,
    pub members: Vec<NwChatMember>,
    pub store: SessionStore,
    pub sender: GossipSender,
    pub receive_handle: JoinHandle<()>,
}

impl Drop for ChatSession {
    fn drop(&mut self) {
        self.receive_handle.abort();
    }
}

impl ChatSession {
    pub async fn spawn(
        chat: NwChat,
        store: SessionStore,
        gossip: &Gossip,
        profile: NwProfile,
    ) -> anyhow::Result<Self> {
        let topic_id = chat.topic_id;

        let bootstrap_ids = chat
            .members
            .iter()
            .filter(|member| {
                member.status == NwChatMemberStatus::Joined
                    && member.contact != profile.contact
            })
            .map(|member| member.contact.endpoint_id)
            .collect();

        let connection = gossip.subscribe(topic_id, bootstrap_ids).await?;
        let (sender, receiver) = connection.split();

        let members = chat.members.clone();
        let receive_store = store.clone();

        let handle = tokio::spawn(async move {
            match receive_loop(receiver, receive_store, topic_id).await {
                Ok(()) => {
                    log::info!("gossip receiver closed for chat: {}", topic_id);
                }
                Err(e) => {
                    log::error!("receive_loop crashed for chat {}: {}", topic_id, e);
                }
            }
        });

        Ok(ChatSession {
            profile,
            topic_id,
            members,
            store,
            sender,
            receive_handle: handle,
        })
    }

    pub async fn send(&self, message_data: MessageData) -> anyhow::Result<()> {
        let content = message_data.content.clone();
        let sent_at = message_data.sent_at as i64;

        let bytes = sign_and_encode(&self.profile.secret_key, message_data)?;
        self.sender.broadcast(bytes.into()).await?;

        self.store
            .record_message(self.profile.contact.endpoint_id, content, sent_at)
            .await?;

        Ok(())
    }
}

pub async fn receive_loop(
    mut receiver: GossipReceiver,
    store: SessionStore,
    topic_id: TopicId,
) -> anyhow::Result<()> {
    while let Some(gossip_event) = receiver.try_next().await? {
        match gossip_event {
            GossipEvent::NeighborUp(_endpoint_id) => {}
            GossipEvent::NeighborDown(_endpoint_id) => {}
            GossipEvent::Received(gossip_message) => match verify_and_decode(gossip_message) {
                Ok(received_message) => {
                    store
                        .record_message(
                            received_message.sender,
                            received_message.content,
                            received_message.sent_at as i64,
                        )
                        .await?;
                }
                Err(e) => {
                    log::warn!(
                        "verify_and_decode failed for chat {}: {}. continuing to process new messages",
                        topic_id,
                        e
                    );
                }
            },
            GossipEvent::Lagged => {}
        }
    }
    Ok(())
}
