use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use iroh::endpoint::{presets, Connection};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::Endpoint;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Handle, Runtime};

use super::{groupchat::ChatSession, CONTROL_ALPN, 
    NwContact, NwProfile, ControlProtocol, ControlMessage};
use super::NwChat;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{};
use crate::ui::UiContact;
use crate::ui::UiChatHeader;

mod ffi;
mod internals;


#[derive(uniffi::Object)]
pub struct NwCore {
    runtime_handle: Handle,
    db_client: DbClient,
    gossip: Gossip,
    router: Router,
    profile: NwProfile,
    chat_connectors: HashMap<TopicId, ChatSession>,
}


#[derive(Debug, thiserror::Error)]
enum SendChatInviteError {
    #[error("timed out waiting for chat invite acceptance")]
    Timeout,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}