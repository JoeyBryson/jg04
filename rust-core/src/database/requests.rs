use crate::network::{NwChat, NwContact, NwMessage, NwProfile};
use tokio::sync::oneshot;
use crate::ui::{UiMessage, UiContact, UiChatHeader, UiChatData};
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


