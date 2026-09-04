pub mod workers;
mod manager;
mod clients;
pub mod sample_data_insertions;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use crate::network::{NwChat, NwContact, NwMessage, NwProfile};
use tokio::sync::oneshot;
use crate::ui::{UiMessage, UiContact, UiChatHeader, UiChatData};

pub struct DbReader{
    rx: mpsc::Receiver<ReadRequest>,
    conn: rusqlite::Connection
}

pub struct DbWriter{
    rx: mpsc::Receiver<WriteRequest>,
    conn: rusqlite::Connection
}

#[derive(uniffi::Object)]
pub struct DbManager {
    reader_tx: mpsc::Sender<ReadRequest>,
    writer_tx: mpsc::Sender<WriteRequest>,
    _reader_handle: JoinHandle<()>,
    _writer_handle: JoinHandle<()>,
}

#[derive(uniffi::Object, Clone)]
pub struct DbClient {
    pub reader_tx: mpsc::Sender<ReadRequest>,
    pub writer_tx: mpsc::Sender<WriteRequest>
}

pub enum ReadRequest {
    ProfileExists {
        reply: oneshot::Sender<anyhow::Result<bool>>,
    },
    GetUiChatHeaders {
        reply: oneshot::Sender<anyhow::Result<Vec<UiChatHeader>>>,
    },
    GetUiChatHeader {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<UiChatHeader>>,
    },
    GetUiChatMembers {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
    },
    GetUiChatMessages {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<UiMessage>>>,
    },
    GetUiChatLastMessage {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<Option<UiMessage>>>,
    },
    GetUiChatData {
        topic_id: String,
        reply: oneshot::Sender<anyhow::Result<UiChatData>>,
    },
    GetUiContacts {
        reply: oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
    },

    GetNwProfile {
        reply: oneshot::Sender<anyhow::Result<NwProfile>>,
    },
    GetNwChats {
        reply: oneshot::Sender<anyhow::Result<Vec<NwChat>>>,
    },
    GetNwChatMembers {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<NwContact>>>,
    },
    GetNwChat {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<NwChat>>,
    },
    GetNwChatMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<anyhow::Result<Vec<NwMessage>>>,
    },
    GetNwMessages {
        reply: oneshot::Sender<anyhow::Result<Vec<NwMessage>>>,
    },
}

pub enum WriteRequest {
    SetNwProfile {
        profile: NwProfile,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    AddNwMessage {
        message: NwMessage,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    AddNwContact {
        contact: NwContact,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    AddNwChat {
        chat: NwChat,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
}


