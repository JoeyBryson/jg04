use tokio::{sync::{mpsc}};
use anyhow::Result;
use tokio::sync::oneshot;

use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr, vec};
use futures_lite::StreamExt;
use iroh::{Endpoint, EndpointAddr, EndpointId, PublicKey, endpoint, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipReceiver}, net::Gossip, proto::{TopicId, topic},
};
use iroh::{endpoint::presets, SecretKey};
use std::println;
use serde::{Deserialize, Serialize};
use tokio::{runtime};
use crate::ffi_error::FfiError;
use std::sync::Arc;


use crate::database::{DbManager};
use thiserror::Error;
// mod iroh_source_sample;
mod profile;
// // mod run;
mod core;
mod chat_manager;

#[derive(Error, Debug, PartialEq)]
pub enum SetupError {
    #[error("profile has not been set up")]
    ProfileNotSet,
    #[error("profile has already been set")]
    ProfileAlreadySet
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NwMessage {
    pub topic_id: TopicId,
    pub from_me: bool,
    pub endpoint_id: Option<EndpointId>,
    pub content: String,
    pub sent_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NwContact {
    pub name: String,
    pub endpoint_id: EndpointId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NwChat {
    pub name: Option<String>,
    pub members: Vec<NwContact>,
    pub topic_id: TopicId,
}

#[derive(Debug, Clone)]

pub struct NwProfile {
    pub secret_key: SecretKey,
}



//#[cfg(test)]
//#[path = "nw/tests.rs"]
//mod tests;

