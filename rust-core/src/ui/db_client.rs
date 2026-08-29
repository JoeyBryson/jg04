use std::result::Result;
use tokio::sync::oneshot;
use std::{path::PathBuf, str::FromStr};
use crate::ui::Arc;
use super::{UiDbClient};
use crate::database::UiDbManager;
use crate::ffi_error::FfiError;

use super::{UiMessage, UiContact, UiChatHeader, UiChatData, UiDbRequest};
use anyhow;

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