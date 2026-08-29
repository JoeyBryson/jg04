use crate::database::UiDbManager;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::sync::Arc;
use anyhow;
use uniffi;
mod db_client;


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
    ProfileExists {
        reply: oneshot::Sender<anyhow::Result<bool>>,
    },
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
    pub worker_tx: mpsc::Sender<UiDbRequest>
}
