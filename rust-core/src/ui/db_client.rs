use std::result::Result;
use tokio::sync::oneshot;
use crate::ui::UiDbManager;

use super::{UiDbClient, UiDbError, UiMessage, UiContact, UiChat, UiChatWithMessages, UiDbRequest};
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

    pub fn get_chats(&self) -> Result<Vec<UiChat>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        // The trailing '?' converts anyhow::Error -> UiDbError automatically via From trait
        Ok(self.send_request(UiDbRequest::GetChats { reply: tx }, rx)?)
    }

    pub fn get_chat(&self, topic_id: Vec<u8>) -> Result<UiChat, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChat { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_members(&self, topic_id: Vec<u8>) -> Result<Vec<UiContact>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatMembers { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_messages(&self, topic_id: Vec<u8>) -> Result<Vec<UiMessage>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatMessages { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_last_message(&self, topic_id: Vec<u8>) -> Result<UiMessage, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatLastMessage { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_with_messages(&self, topic_id: Vec<u8>) -> Result<UiChatWithMessages, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatWithMessages { topic_id, reply: tx }, rx)?)
    }

    pub fn get_contacts(&self) -> Result<Vec<UiContact>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetContacts { reply: tx }, rx)?)
    }
}