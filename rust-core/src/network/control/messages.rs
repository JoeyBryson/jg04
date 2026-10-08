use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};

use crate::network::NwChat;

#[derive(Debug, Serialize, Deserialize)]
pub(in crate::network) enum ControlRequest {
    ChatInvite { chat: NwChat },
}

#[derive(Debug, Serialize, Deserialize)]
pub(in crate::network) enum ControlResponse {
    ChatInviteAccepted { topic_id: TopicId },
}
