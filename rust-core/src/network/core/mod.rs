use iroh::protocol::Router;
use iroh_gossip::net::Gossip;
use tokio::runtime::Handle;

use super::NwChat;
use super::{NwProfile, groupchat::ChatSessionManager};
use crate::database::client::DbClient;

mod ffi;
mod internals;

#[derive(uniffi::Object)]
pub struct NwCore {
    runtime_handle: Handle,
    db_client: DbClient,
    gossip: Gossip,
    router: Router,
    profile: NwProfile,
    chat_session_manager: ChatSessionManager,
}

#[derive(Debug, thiserror::Error)]
enum SendChatInviteError {
    #[error("timed out waiting for chat invite acceptance")]
    Timeout,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
