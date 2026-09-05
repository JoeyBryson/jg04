use iroh::EndpointId;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender}, net::Gossip, proto::TopicId,
};
use super::NwChat;
use crate::database::client::DbClient;
use crate::network::NwContact;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { active_peers: usize },
    Reconnecting { attempt: u32 },
}

pub struct NwChatConnector {
    pub db_client: DbClient,
    pub topic_id: TopicId,
    pub members: Vec<NwContact>,
    pub sender: Option<GossipSender>,
    pub receiver: Option<GossipReceiver>,
    pub state: ConnectionState,
}

impl NwChatConnector {
    /// 1. Decoupled creation: Initializes state locally without requiring network I/O.
    pub fn new(chat: NwChat, db_client: DbClient) -> Self {
        todo!("Construct connector in Disconnected state without joining gossip network")
    }

    /// 1 & 4. Joins or re-joins the gossip swarm asynchronously with retry/backoff logic.
    pub async fn connect(&mut self, gossip: &Gossip) -> anyhow::Result<()> {
        todo!("Update state to Connecting, join gossip, split channels, update state to Connected")
    }

    /// 4. Disconnects existing swarm handles gracefully on network drop or app pause.
    pub async fn disconnect(&mut self) {
        todo!("Drop sender/receiver channels and set state to Disconnected")
    }

    /// 2. Queries active peer count for this topic and updates inner state machine.
    pub async fn sync_peer_state(&mut self, gossip: &Gossip) {
        todo!("Inspect active swarm connections, update active_peers, transition state if needed")
    }

    /// 2. Returns current connection state to be surfaced over FFI.
    pub fn state(&self) -> ConnectionState {
        todo!("Return current connection status")
    }
}