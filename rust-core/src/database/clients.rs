use std::result::Result;
use tokio::sync::oneshot;
use std::{path::PathBuf, str::FromStr};
use crate::database::UiDbManager;
use crate::ffi_error::FfiError;

use crate::ui::{UiMessage, UiContact, UiChatHeader, UiChatData};
use anyhow;
use iroh::EndpointId;
use anyhow::{Context};
use hex;
use iroh_gossip::TopicId;

use crate::network::{NwChat, NwContact, NwMessage};
use crate::{notifications::{UiEvent, emit_ui_event}};
use crate::network::{NwProfile};
use super::{NwDbClient, NwDbRequest, UiDbClient, UiDbRequest};

impl UiDbClient {
    fn send_request<T>(
        &self,
        request: UiDbRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.worker_tx
            .blocking_send(request)?;

        rx.blocking_recv()?
    }
}

#[uniffi::export]
impl UiDbClient {

    pub fn profile_exists(&self) -> Result<bool, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::ProfileExists { reply: tx }, rx)?)
    }

    pub fn get_chat_headers(&self) -> Result<Vec<UiChatHeader>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatHeaders { reply: tx }, rx)?)
    }

    pub fn get_chat_header(&self, topic_id: String) -> Result<UiChatHeader, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatHeader { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_members(&self, topic_id: String) -> Result<Vec<UiContact>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatMembers { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_messages(&self, topic_id: String) -> Result<Vec<UiMessage>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatMessages { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_last_message(&self, topic_id: String) -> Result<Option<UiMessage>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatLastMessage { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_data(&self, topic_id: String) -> Result<UiChatData, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatData { topic_id, reply: tx }, rx)?)
    }

    pub fn get_contacts(&self) -> Result<Vec<UiContact>, FfiError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetContacts { reply: tx }, rx)?)
    }
}

impl NwDbClient {

        fn send_request_sync<T>(
        &self,
        request: NwDbRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.worker_tx
            .blocking_send(request)?;

        rx.blocking_recv()?
    }

    async fn send_request_async<T>(
        &self,
        request: NwDbRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.worker_tx
            .send(request)
            .await?;

        let res = rx
            .await??;

        Ok(res)
    }

    pub async fn set_profile(&self, profile: NwProfile) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::SetProfile { profile, reply: tx }, rx).await
    }

    pub async fn get_profile(&self) -> anyhow::Result<NwProfile> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::GetProfile { reply: tx }, rx).await
    }

    pub async fn get_chats(&self) -> anyhow::Result<Vec<NwChat>> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::GetChats { reply: tx }, rx).await
    }

    pub async fn add_message(&self, message: NwMessage) -> anyhow::Result<()> {
        let topic_id = message.topic_id.clone();
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::AddMessage { message, reply: tx }, rx).await?;
        emit_ui_event(UiEvent::ChatDataChanged { topic_id: hex::encode(topic_id) });
        emit_ui_event(UiEvent::ChatHeadersChanged);
        Ok(())
    }

    pub async fn add_contact(&self, contact: NwContact) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::AddContact { contact, reply: tx }, rx).await?;
        emit_ui_event(UiEvent::ContactsChanged);
        Ok(())
    }

    pub async fn add_chat(&self, chat: NwChat) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::AddChat { chat, reply: tx }, rx).await?;
        emit_ui_event(UiEvent::ChatHeadersChanged);
        Ok(())
    }

    pub async fn get_chat_members(&self, topic_id: Vec<u8>) -> anyhow::Result<Vec<NwContact>> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::GetChatMembers { topic_id, reply: tx }, rx).await
    }

    pub async fn get_chat(&self, topic_id: Vec<u8>) -> anyhow::Result<NwChat> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::GetChat { topic_id, reply: tx }, rx).await
    }

    pub async fn get_chat_messages(&self, topic_id: Vec<u8>) -> anyhow::Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::GetChatMessages { topic_id, reply: tx }, rx).await
    }

    pub async fn get_messages(&self) -> anyhow::Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();
        self.send_request_async(NwDbRequest::GetMessages { reply: tx }, rx).await
    }
}