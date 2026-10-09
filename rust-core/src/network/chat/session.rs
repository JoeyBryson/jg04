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
            .collect::<Vec<_>>();

        let topic_id = store.topic_id();

        log::info!(
            "[CHAT-SESSION] subscribing to topic={} with {} bootstrap peers",
            topic_id,
            bootstrap_ids.len()
        );
        if !bootstrap_ids.is_empty() {
            let peers = bootstrap_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            log::debug!("[CHAT-SESSION] topic={} bootstrap peers=[{}]", topic_id, peers);
        }

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
        let topic_id = self.store.topic_id();

        log::debug!(
            "[CHAT-SESSION] send start: topic={}, sender={}, sent_at={}, content_len={}",
            topic_id,
            sender,
            message_data.sent_at,
            content.len()
        );

        let bytes = sign_and_encode(&profile.secret_key, message_data)?;
        log::trace!(
            "[CHAT-SESSION] encoded message: topic={}, sender={}, bytes={}",
            topic_id,
            sender,
            bytes.len()
        );
        self.sender.broadcast(bytes.into()).await?;
        log::debug!(
            "[CHAT-SESSION] gossip broadcast complete: topic={}, sender={}",
            topic_id,
            sender
        );

        self.store
            .record_message(sender, content, sent_at)
            .await?;

        log::debug!(
            "[CHAT-SESSION] send persisted locally: topic={}, sender={}, sent_at={}",
            topic_id,
            sender,
            sent_at
        );

        Ok(())
    }
}

pub async fn receive_loop(
    mut receiver: GossipReceiver,
    store: Arc<SessionStore>,
) -> anyhow::Result<()> {
    while let Some(gossip_event) = receiver.try_next().await? {
        match gossip_event {
            GossipEvent::NeighborUp(endpoint_id) => {
                log::info!(
                    "[CHAT-SESSION] neighbor up: topic={}, peer={}",
                    store.topic_id(),
                    endpoint_id
                );
            }
            GossipEvent::NeighborDown(endpoint_id) => {
                log::info!(
                    "[CHAT-SESSION] neighbor down: topic={}, peer={}",
                    store.topic_id(),
                    endpoint_id
                );
            }
            GossipEvent::Received(gossip_message) => match verify_and_decode(gossip_message) {
                Ok(received_message) => {
                    log::debug!(
                        "[CHAT-SESSION] received message: topic={}, sender={}, sent_at={}, content_len={}",
                        store.topic_id(),
                        received_message.sender,
                        received_message.sent_at,
                        received_message.content.len()
                    );

                    store
                        .record_message(
                            received_message.sender,
                            received_message.content,
                            received_message.sent_at as i64,
                        )
                        .await?;

                    log::debug!(
                        "[CHAT-SESSION] persisted received message: topic={}, sender={}",
                        store.topic_id(),
                        received_message.sender
                    );
                }
                Err(e) => {
                    log::warn!(
                        "verify_and_decode failed for chat {}: {}. continuing to process new messages",
                        store.topic_id(),
                        e
                    );
                }
            },
            GossipEvent::Lagged => {
                log::warn!("[CHAT-SESSION] receiver lagged for topic={}", store.topic_id());
            }
        }
    }

    log::info!("[CHAT-SESSION] receive loop ended for topic={}", store.topic_id());

    Ok(())
}
