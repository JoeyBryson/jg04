
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;
use iroh::SecretKey;
use serde::{Deserialize, Serialize};
use crate::ui::{UiContact, UiChatHeader};

use thiserror::Error;
// mod iroh_source_sample;
mod profile;
// // mod run;
mod core;
mod groupchat;
mod events;
mod signed_message;
mod control_protocol;

pub(super) const CONTROL_ALPN: &[u8] = b"iroh-example/echo/0";

#[derive(Error, Debug, PartialEq)]
pub enum SetupError {
    #[error("profile has not been set up")]
    ProfileNotSet,
    #[error("profile has already been set")]
    ProfileAlreadySet
}
use control_protocol::ControlProtocol;
use control_protocol::ControlMessage;
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
pub struct NwChat {
    pub name: Option<String>,
    pub members: Vec<NwContact>,
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
    pub contact: NwContact
}

impl From<UiContact> for NwContact {
    fn from(contact: UiContact) -> Self {
        Self {
            name: contact.name,
            endpoint_id: contact.endpoint_id.parse().unwrap(),
        }
    }
}

impl From<UiChatHeader> for NwChat {
    fn from(chat: UiChatHeader) -> Self {
        Self {
            name: chat.name,
            members: chat.members.into_iter().map(Into::into).collect(),
            topic_id: chat.topic_id.parse().unwrap(),
        }
    }
}

//#[cfg(test)]
//#[path = "nw/tests.rs"]
//mod tests;

