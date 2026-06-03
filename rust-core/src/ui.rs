use crate::db::DbManager;
use crate::nw;
use crate::notifications::UiEventListener;
use tokio::{sync::{mpsc}};
use std::result::Result;
use std::sync::OnceLock;
use std::thread::JoinHandle;

use tokio::sync::oneshot;
// use uniffi;
use std::sync::{Arc, Mutex};
use anyhow;
use uniffi;
mod db_client;
mod db_manager;

use uniffi::Enum;


#[derive(uniffi::Error, Debug)]
pub enum UiDbError {
    InternalError { msg: String },
}

impl std::fmt::Display for UiDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiDbError::InternalError { msg } => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<anyhow::Error> for UiDbError {
    fn from(err: anyhow::Error) -> Self {
        UiDbError::InternalError {
            msg: err.to_string(),
        }
    }
}

impl std::error::Error for UiDbError {}

#[derive(uniffi::Enum)]
pub enum UiSender {
    Me,
    Other(UiContact)
}

#[derive(uniffi::Record)]
pub struct UiContact {
    pub name: String,
    pub endpoint_id: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct UiChat {
    pub name: Option<String>,
    pub members: Vec<UiContact>,
    pub topic_id: Vec<u8>,
    pub last_message: UiMessage
}

#[derive(uniffi::Record)]
pub struct UiMessage {
    pub sender: UiSender,
    pub content: String,
    pub sent_at: i64,
}
#[derive(uniffi::Record)]
pub struct UiChatWithMessages {
    pub chat: UiChat,
    pub messages: Vec<UiMessage>
}


pub enum UiDbRequest {
    GetChats {
        reply: oneshot::Sender<anyhow::Result<Vec<UiChat>>>,
    },
    GetChat {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<UiChat>>,
    },
    GetChatMembers {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
    },
    GetChatMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<UiMessage>>>,
    },
    GetChatLastMessage {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<UiMessage>>,
    },
    GetChatWithMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<UiChatWithMessages>>,
    },
    GetContacts {
        reply: oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
    },
}

#[derive(uniffi::Object)]
pub struct UiDbClient {
    worker_tx: mpsc::Sender<UiDbRequest>
}

//Uniffi wrapper since it cannot handle generic definitions
#[derive(uniffi::Object)]
pub struct UiDbManager {
    inner: DbManager<UiDbRequest>
}
