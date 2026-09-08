use futures_lite::StreamExt;
use iroh::{EndpointId, SecretKey};
use iroh_gossip::{
    api::{GossipReceiver, GossipSender}, net::Gossip, proto::TopicId,
};
use iroh_gossip::api::Event as GossipEvent;
use super::{NwChat, NwMessage,  events::NwEvent, signed_message::{verify_and_decode, sign_and_encode}};
use crate::{database::client::DbClient, network::signed_message::MessageData};
use crate::network::NwContact;
use std::time::SystemTime;
use tokio::task::JoinHandle;

pub struct NwChatConnector {
    pub secret_key: SecretKey,
    pub topic_id: TopicId,
    pub db_client: DbClient,
    pub members: Vec<NwContact>,
    pub sender: GossipSender,
    pub receive_handle: JoinHandle<()>,
}


impl Drop for NwChatConnector {
    fn drop(&mut self) {
        self.receive_handle.abort();
    }
}


impl NwChatConnector {
    pub async fn spawn(chat: NwChat, db_client: DbClient, gossip: &Gossip, secret_key: SecretKey) -> anyhow::Result<Self> {
        let topic_id = chat.topic_id;
        let members = chat.members;
        let bootstrap_ids = members
            .iter()
            .map(|member| member.endpoint_id)
            .collect();

        let connection = gossip.subscribe(topic_id, bootstrap_ids).await?;
        let (sender, receiver) = connection.split();

        let db_client_clone = db_client.clone();

        let handle = tokio::spawn(async move {
            match receive_loop(receiver, db_client_clone, topic_id).await {
                Ok(()) => {
                    log::info!("gossip receiver closed for chat: {}", topic_id);
                }
                Err(e) => {
                    log::error!("receive_loop crashed for chat {}: {}", topic_id, e);
                }
            }
        });

        Ok(NwChatConnector {
            topic_id,
            db_client,
            members,
            sender,
            receive_handle: handle,
            secret_key
        })
    }

    pub async fn send(&self, content: String) -> anyhow::Result<()>{
        let sent_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message_data = MessageData{content, sent_at};
        let bytes = sign_and_encode(&self.secret_key, message_data.clone())?;
        self.sender.broadcast(bytes.into()).await?;
        self.db_client.add_nw_message(
            NwMessage { topic_id: self.topic_id, 
                from_me: true, 
                endpoint_id: None, 
                content: message_data.content, 
                sent_at: message_data.sent_at as i64}
        ).await?;
        
        Ok(())
    }
}



pub async fn receive_loop(
    mut receiver: GossipReceiver,
    db_client: DbClient,
    topic_id: TopicId,
) -> anyhow::Result<()> {
    while let Some(gossip_event) = receiver.try_next().await? {
        match gossip_event {
            GossipEvent::NeighborUp(_endpoint_id) => {}
            GossipEvent::NeighborDown(_endpoint_id) => {}
            GossipEvent::Received(gossip_message) => {
                match verify_and_decode(gossip_message) {
                    Ok(received_message) => {
                        db_client
                            .add_nw_message(NwMessage {
                                topic_id,
                                from_me: false,
                                endpoint_id: Some(received_message.sender),
                                content: received_message.content,
                                sent_at: received_message.sent_at as i64,
                            })
                            .await?;
                    }
                    Err(e) => {
                        log::warn!(
                            "verify_and_decode failed for chat {}: {}. continuing to process new messages",
                            topic_id,
                            e
                        );
                    }
                }
            }
            GossipEvent::Lagged => {}
        }
    }
    Ok(())
}

//     /// 1 & 4. Joins or re-joins the gossip swarm asynchronously with retry/backoff logic.
//     pub async fn connect(&mut self, gossip: &Gossip) -> anyhow::Result<()> {
//         todo!("Update state to Connecting, join gossip, split channels, update state to Connected")
//     }

//     /// 4. Disconnects existing swarm handles gracefully on network drop or app pause.
//     pub async fn disconnect(&mut self) {
//         todo!("Drop sender/receiver channels and set state to Disconnected")
//     }
//     // /// 2. Returns current connection state to be surfaced over FFI.
//     // pub fn state(&self) -> ConnectionState {
//     //     todo!("Return current connection status")
//     // }
// }