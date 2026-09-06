use futures_lite::StreamExt;
use iroh::EndpointId;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender}, net::Gossip, proto::TopicId,
};
use super::{NwChat, events::NwEvent};
use crate::database::client::DbClient;
use crate::network::NwContact;


// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum ConnectionState {
//     Disconnected,
//     Connecting,
//     Connected { active_peers: usize },
//     Reconnecting { attempt: u32 },
// }

pub struct NwChatConnector {
    pub topic_id: TopicId,
    pub db_client: DbClient,
    pub members: Vec<NwContact>,
    pub sender: GossipSender,
    pub receiver: GossipReceiver,
    // pub state: ConnectionState,
}

impl NwChatConnector {
    pub async fn spawn(chat: NwChat, db_client: DbClient, gossip: &Gossip) -> anyhow::Result<Self> {
        let topic_id = chat.topic_id;
        let members = chat.members;
        let bootstrap_ids = members
            .iter()
            .map(|member| member.endpoint_id)
            .collect();

        let connection = gossip.subscribe(topic_id, bootstrap_ids).await?;
        let (sender, receiver) = connection.split();
        Ok(
            NwChatConnector { topic_id, db_client, members , sender, receiver }
        )
    }

    pub async fn receive_loop(receiver: &mut GossipReceiver) -> anyhow::Result<()> {
        while let Some(gossip_event) = receiver.try_next().await? {
            let nw_event = NwEvent::try_from(gossip_event)?;
            match nw_event {
                
                
            }
        }

    Ok(())
}
    /// 1 & 4. Joins or re-joins the gossip swarm asynchronously with retry/backoff logic.
    pub async fn connect(&mut self, gossip: &Gossip) -> anyhow::Result<()> {
        todo!("Update state to Connecting, join gossip, split channels, update state to Connected")
    }

    /// 4. Disconnects existing swarm handles gracefully on network drop or app pause.
    pub async fn disconnect(&mut self) {
        todo!("Drop sender/receiver channels and set state to Disconnected")
    }
    // /// 2. Returns current connection state to be surfaced over FFI.
    // pub fn state(&self) -> ConnectionState {
    //     todo!("Return current connection status")
    // }
}