use tokio::{sync::{mpsc}};
use anyhow::Result;
use tokio::sync::oneshot;

use crate::database::{NwDbManager};
use thiserror::Error;

mod db_client;
// mod iroh_source_sample;
// // mod profile;
// // mod run;
mod core;
// mod chat_manager;

#[derive(Error, Debug, PartialEq)]
pub enum SetupError {
    #[error("profile has not been set up")]
    ProfileNotSet,
    #[error("profile has already been set")]
    ProfileAlreadySet
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NwMessage {
    pub topic_id: Vec<u8>,
    pub from_me: bool,
    pub endpoint_id: Option<Vec<u8>>,
    pub content: String,
    pub sent_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NwContact {
    pub name: String,
    pub endpoint_id: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NwChat {
    pub name: Option<String>,
    pub members: Vec<NwContact>,
    pub topic_id: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]

pub struct NwProfile {
    pub secret_key: Vec<u8>,
}

pub enum NwDbRequest {
    SetProfile {
        profile: NwProfile,
        reply: oneshot::Sender<Result<()>>
    },
    GetProfile {
        reply: oneshot::Sender<Result<NwProfile>>
    },
    AddMessage {
        message: NwMessage,
        reply: oneshot::Sender<Result<()>>,
    },
    AddContact {
        contact: NwContact,
        reply: oneshot::Sender<Result<()>>,
    },
    AddChat {
        chat: NwChat,
        reply: oneshot::Sender<Result<()>>,
    },
    GetChats {
        reply: oneshot::Sender<Result<Vec<NwChat>>>,
    },
    GetChatMembers {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<NwContact>>>,
    },
    GetChat {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<NwChat>>,
    },
    GetChatMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<NwMessage>>>,
    },
    GetMessages {
        reply: oneshot::Sender<Result<Vec<NwMessage>>>,
    },
}

pub struct NwDbClient {
    worker_tx: mpsc::Sender<NwDbRequest>
}


//#[cfg(test)]
//#[path = "nw/tests.rs"]
//mod tests;

