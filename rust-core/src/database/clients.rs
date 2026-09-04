use std::result::Result;
use tokio::sync::oneshot;

use crate::ffi_error::FfiError;
use crate::network::{NwChat, NwContact, NwMessage, NwProfile};
use crate::notifications::{emit_ui_event, UiEvent};
use crate::ui::{UiMessage, UiContact, UiChatHeader, UiChatData};

use super::{DbClient, ReadRequest, WriteRequest};

impl DbClient {
    fn send_read_request_sync<T>(
        &self,
        request: ReadRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.reader_tx.blocking_send(request)?;
        rx.blocking_recv()?
    }

    async fn send_read_request_async<T>(
        &self,
        request: ReadRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.reader_tx.send(request).await?;
        rx.await?
    }

    fn send_write_request_sync<T>(
        &self,
        request: WriteRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.writer_tx.blocking_send(request)?;
        rx.blocking_recv()?
    }

    async fn send_write_request_async<T>(
        &self,
        request: WriteRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.writer_tx.send(request).await?;
        rx.await?
    }
}

#[uniffi::export]
impl DbClient {
    pub fn profile_exists(&self) -> Result<bool, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::ProfileExists { reply: tx }, rx)?)
    }

    pub fn get_ui_chat_headers(&self) -> Result<Vec<UiChatHeader>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiChatHeaders { reply: tx }, rx)?)
    }

    pub fn get_ui_chat_header(&self, topic_id: String) -> Result<UiChatHeader, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiChatHeader { topic_id, reply: tx }, rx)?)
    }

    pub fn get_ui_chat_members(&self, topic_id: String) -> Result<Vec<UiContact>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiChatMembers { topic_id, reply: tx }, rx)?)
    }

    pub fn get_ui_chat_messages(&self, topic_id: String) -> Result<Vec<UiMessage>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiChatMessages { topic_id, reply: tx }, rx)?)
    }

    pub fn get_ui_chat_last_message(&self, topic_id: String) -> Result<Option<UiMessage>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiChatLastMessage { topic_id, reply: tx }, rx)?)
    }

    pub fn get_ui_chat_data(&self, topic_id: String) -> Result<UiChatData, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiChatData { topic_id, reply: tx }, rx)?)
    }

    pub fn get_ui_contacts(&self) -> Result<Vec<UiContact>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_read_request_sync( ReadRequest::GetUiContacts { reply: tx }, rx)?)
    }
}

impl DbClient {
    pub async fn set_profile(&self, profile: NwProfile) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.send_write_request_async( WriteRequest::SetNwProfile { profile, reply: tx }, rx).await
    }

    pub async fn get_nw_profile(&self) -> anyhow::Result<NwProfile> {
        let (tx, rx) = oneshot::channel();
        self.send_read_request_async( ReadRequest::GetNwProfile { reply: tx }, rx).await
    }

    pub async fn get_nw_chats(&self) -> anyhow::Result<Vec<NwChat>> {
        let (tx, rx) = oneshot::channel();
        self.send_read_request_async( ReadRequest::GetNwChats { reply: tx }, rx).await
    }

    pub async fn add_nw_message(&self, message: NwMessage) -> anyhow::Result<()> {
        let topic_id = message.topic_id;
        let (tx, rx) = oneshot::channel();
        self.send_write_request_async( WriteRequest::AddNwMessage { message, reply: tx }, rx).await?;
        emit_ui_event(UiEvent::ChatDataChanged { topic_id: hex::encode(topic_id) });
        emit_ui_event(UiEvent::ChatHeadersChanged);
        Ok(())
    }

    pub async fn add_nw_contact(&self, contact: NwContact) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.send_write_request_async( WriteRequest::AddNwContact { contact, reply: tx }, rx).await?;
        emit_ui_event(UiEvent::ContactsChanged);
        Ok(())
    }

    pub async fn add_nw_chat(&self, chat: NwChat) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.send_write_request_async( WriteRequest::AddNwChat { chat, reply: tx }, rx).await?;
        emit_ui_event(UiEvent::ChatHeadersChanged);
        Ok(())
    }

    pub async fn get_nw_chat_members(&self, topic_id: Vec<u8>) -> anyhow::Result<Vec<NwContact>> {
        let (tx, rx) = oneshot::channel();
        self.send_read_request_async( ReadRequest::GetNwChatMembers { topic_id, reply: tx }, rx).await
    }

    pub async fn get_nw_chat(&self, topic_id: Vec<u8>) -> anyhow::Result<NwChat> {
        let (tx, rx) = oneshot::channel();
        self.send_read_request_async( ReadRequest::GetNwChat { topic_id, reply: tx }, rx).await
    }

    pub async fn get_nw_chat_messages(&self, topic_id: Vec<u8>) -> anyhow::Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();
        self.send_read_request_async( ReadRequest::GetNwChatMessages { topic_id, reply: tx }, rx).await
    }

    pub async fn get_nw_messages(&self) -> anyhow::Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();
        self.send_read_request_async( ReadRequest::GetNwMessages { reply: tx }, rx).await
    }
}