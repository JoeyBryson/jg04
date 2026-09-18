use crate::ui::{UiChatHeader, UiContact};
use iroh::EndpointId;
use iroh::SecretKey;
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};

use thiserror::Error;
// mod iroh_source_sample;
mod profile;
// // mod run;
mod control_protocol;
mod core;
mod events;
mod groupchat;
mod signed_message;

pub(super) const CONTROL_ALPN: &[u8] = b"iroh-example/echo/0";

#[derive(Error, Debug, PartialEq)]
pub enum SetupError {
    #[error("profile has not been set up")]
    ProfileNotSet,
    #[error("profile has already been set")]
    ProfileAlreadySet,
}
use control_protocol::ControlMessage;
use control_protocol::ControlProtocol;
use groupchat::ChatSessionManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NwMessage {
    pub topic_id: TopicId,
    pub from_me: bool,
    pub endpoint_id: Option<EndpointId>,
    pub content: String,
    pub sent_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NwChatMemberStatus{
    Pending,
    Joined
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NwChatMember {
    contact: NwContact,
    status: NwChatMemberStatus
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NwChat {
    pub name: Option<String>,
    pub members: Vec<NwChatMember>,
    pub topic_id: TopicId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NwContact {
    pub name: String,
    pub endpoint_id: EndpointId,
}

#[derive(Debug, Clone)]

pub struct NwProfile {
    pub secret_key: SecretKey,
    pub contact: NwContact,
}

impl From<UiContact> for NwContact {
    fn from(contact: UiContact) -> Self {
        Self {
            name: contact.name,
            endpoint_id: contact.endpoint_id.parse().unwrap(),
        }
    }
}


//#[cfg(test)]
//#[path = "nw/tests.rs"]
//mod tests;
