use crate::db::UiDbManager;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::sync::Arc;
use anyhow;
use uniffi;
mod db;


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
    pub endpoint_id: String,
}

#[derive(uniffi::Record)]
pub struct UiChatHeader {
    pub name: Option<String>,
    pub members: Vec<UiContact>,
    pub topic_id: String,
    pub last_message: Option<UiMessage>
}
#[derive(uniffi::Record)]
pub struct UiMessage {
    pub sender: UiSender,
    pub content: String,
    pub sent_at: i64,
}
#[derive(uniffi::Record)]
pub struct UiChatData {
    pub chat: UiChatHeader,
    pub messages: Vec<UiMessage>
}


pub enum UiDbRequest {
    GetChatHeaders {
        reply: oneshot::Sender<anyhow::Result<Vec<UiChatHeader>>>,
    },
    GetChatHeader {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<UiChatHeader>>,
    },
    GetChatMembers {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
    },
    GetChatMessages {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<UiMessage>>>,
    },
    GetChatLastMessage {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<Option<UiMessage>>>,
    },
    GetChatData {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<UiChatData>>,
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
pub struct UiDbManagerUniffiObject {
    inner: UiDbManager
}
