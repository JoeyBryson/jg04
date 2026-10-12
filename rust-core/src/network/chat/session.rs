use crate::network::messaging::signed::{MessageData, sign_and_encode, verify_and_decode};
use crate::network::persistence::SessionStore;
use crate::network::{NwChatMemberStatus, short_topic_id};
use futures_lite::StreamExt;
use iroh_gossip::api::Event as GossipEvent;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender},
    net::Gossip,
};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::Instrument;

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
    pub async fn spawn(store: SessionStore, gossip: &Gossip) -> anyhow::Result<Self> {
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

        let connection = gossip.subscribe(topic_id, bootstrap_ids).await?;
        let (sender, receiver) = connection.split();

        let receive_store = Arc::clone(&store);

        let receive_span = tracing::info_span!("chat_session", topic = %short_topic_id(&topic_id));
        let handle = tokio::spawn(
            async move {
                match receive_loop(receiver, receive_store).await {
                    Ok(()) => {}
                    Err(error) => {
                        tracing::error!(error = %error, "receive loop failed");
                    }
                }
            }
            .instrument(receive_span),
        );

        Ok(ChatSession {
            store,
            sender,
            receive_handle: handle,
        })
    }

    pub async fn send(&self, message_data: MessageData) -> anyhow::Result<()> {
        let profile = self.store.profile();
        let (bytes, persisted_message) =
            sign_and_encode(&profile.secret_key, self.store.topic_id(), message_data)?;
        self.store.record_message(persisted_message).await?;
        self.sender.broadcast(bytes.into()).await?;

        Ok(())
    }
}

pub async fn receive_loop(
    mut receiver: GossipReceiver,
    store: Arc<SessionStore>,
) -> anyhow::Result<()> {
    while let Some(gossip_event) = receiver.try_next().await? {
        match gossip_event {
            GossipEvent::NeighborUp(_) => {}
            GossipEvent::NeighborDown(_) => {}
            GossipEvent::Received(gossip_message) => {
                match verify_and_decode(gossip_message, store.topic_id()) {
                    Ok(received_message) => {
                        if let Err(error) = store.record_message(received_message).await {
                            tracing::warn!(error = %error, "ignoring message from a non-joined chat member");
                        }
                    }
                    Err(error) => {
                        tracing::warn!(error = %error, "ignoring invalid gossip message");
                    }
                }
            }
            GossipEvent::Lagged => {
                tracing::warn!("message receiver lagged");
            }
        }
    }

    Ok(())
}
