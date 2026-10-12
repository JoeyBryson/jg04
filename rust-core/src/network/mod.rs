use iroh_gossip::proto::TopicId;
use thiserror::Error;
mod api;
mod application;
mod chat;
mod control;
mod messaging;
mod model;
mod persistence;

pub use api::{NwCore, NwCoreInterface};
pub use messaging::NwEvent;
pub use model::{NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile};

pub(super) const CONTROL_ALPN: &[u8] = b"iroh-example/echo/0";
pub(crate) const MAX_CHAT_MESSAGE_CONTENT_BYTES: usize = 3800;

pub(super) fn short_topic_id(topic_id: &TopicId) -> String {
    let id = topic_id.to_string();
    id[..10].to_owned()
}

#[derive(Error, Debug, PartialEq)]
pub enum SetupError {
    #[error("profile has not been set up")]
    ProfileNotSet,
    #[error("profile has already been set")]
    ProfileAlreadySet,
}
