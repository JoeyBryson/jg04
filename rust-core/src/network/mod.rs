use thiserror::Error;
mod api;
mod application;
mod chat;
mod control;
mod messaging;
mod model;
mod persistence;

pub use api::{NwCore, NwInterface};
pub use messaging::NwEvent;
pub use model::{NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile};

pub(super) const CONTROL_ALPN: &[u8] = b"iroh-example/echo/0";

#[derive(Error, Debug, PartialEq)]
pub enum SetupError {
    #[error("profile has not been set up")]
    ProfileNotSet,
    #[error("profile has already been set")]
    ProfileAlreadySet,
}

