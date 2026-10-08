use crate::network::messaging::signed::{sign_and_encode, verify_and_decode, MessageData};
use crate::network::persistence::SessionStore;
use crate::network::NwChatMemberStatus;
use futures_lite::StreamExt;
use iroh_gossip::api::Event as GossipEvent;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender},
    net::Gossip,
};
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct ChatSession {
    pub store: Arc<SessionStore>,
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
        store: SessionStore,
        gossip: &Gossip,
    ) -> anyhow::Result<Self> {
        let store = Arc::new(store);
        let bootstrap_ids = store
            .chat()
            .members
            .iter()
            .filter(|member| {
                member.status == NwChatMemberStatus::Joined
                    && member.contact != store.profile().contact
            })
            .map(|member| member.contact.endpoint_id)
            .collect();

        let topic_id = store.topic_id();

        let connection = gossip.subscribe(topic_id, bootstrap_ids).await?;
        let (sender, receiver) = connection.split();

        let receive_store = Arc::clone(&store);

        let handle = tokio::spawn(async move {
            match receive_loop(receiver, receive_store).await {
                Ok(()) => {
                    log::info!("gossip receiver closed for chat: {}", topic_id);
                }
                Err(e) => {
                    log::error!("receive_loop crashed for chat {}: {}", topic_id, e);
                }
            }
        });

        Ok(ChatSession {
            store,
            sender,
            receive_handle: handle,
        })
    }

    pub async fn send(&self, message_data: MessageData) -> anyhow::Result<()> {
        let content = message_data.content.clone();
        let sent_at = message_data.sent_at as i64;

        let profile = self.store.profile();
        let sender = profile.contact.endpoint_id;
        let bytes = sign_and_encode(&profile.secret_key, message_data)?;
        self.sender.broadcast(bytes.into()).await?;

        self.store
            .record_message(sender, content, sent_at)
            .await?;

        Ok(())
    }
}

pub async fn receive_loop(
    mut receiver: GossipReceiver,
    store: Arc<SessionStore>,
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
                        store.topic_id(),
                        e
                    );
                }
            },
            GossipEvent::Lagged => {}
        }
    }
    Ok(())
}
