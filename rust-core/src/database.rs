mod requests;
pub mod workers;
mod manager;
mod clients;
pub mod sample_data_insertions;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use crate::network::{NwChat, NwContact, NwMessage, NwProfile};
use tokio::sync::oneshot;
use crate::ui::{UiMessage, UiContact, UiChatHeader, UiChatData};

pub struct UiDbWorker{
    worker_rx: mpsc::Receiver<UiDbRequest>,
    conn: rusqlite::Connection
}

pub struct NwDbWorker{
    worker_rx: mpsc::Receiver<NwDbRequest>,
    conn: rusqlite::Connection
}

#[derive(uniffi::Object)]
pub struct UiDbManager {
    worker_tx: mpsc::Sender<UiDbRequest>,
    _join_handle: JoinHandle<()>,
}
#[derive(uniffi::Object)]
pub struct NwDbManager {
    worker_tx: mpsc::Sender<NwDbRequest>,
    _join_handle: JoinHandle<()>,
}

#[derive(uniffi::Object, Clone)]
pub struct UiDbClient {
    pub worker_tx: mpsc::Sender<UiDbRequest>
}


#[derive(uniffi::Object, Clone)]
pub struct NwDbClient {
    pub worker_tx: mpsc::Sender<NwDbRequest>
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

pub enum NwDbRequest {
    SetProfile {
        profile: NwProfile,
        reply: oneshot::Sender<anyhow::Result<()>>
    },
    GetProfile {
        reply: oneshot::Sender<anyhow::Result<NwProfile>>
    },
    AddMessage {
        message: NwMessage,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    AddContact {
        contact: NwContact,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    AddChat {
        chat: NwChat,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    GetChats {
        reply: oneshot::Sender<anyhow::Result<Vec<NwChat>>>,
    },
    GetChatMembers {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<NwContact>>>,
    },
    GetChat {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<NwChat>>,
    },
    GetChatMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<NwMessage>>>,
    },
    GetMessages {
        reply: oneshot::Sender<anyhow::Result<Vec<NwMessage>>>,
    },
}


