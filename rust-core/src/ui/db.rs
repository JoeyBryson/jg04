use std::result::Result;
use tokio::sync::oneshot;
use std::{path::PathBuf, str::FromStr};
use crate::ui::Arc;
use super::{UiDbManagerUniffiObject, UiDbError, UiDbClient};
use crate::db::UiDbManager;

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

    pub fn get_chat_headers(&self) -> Result<Vec<UiChatHeader>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        // The trailing '?' converts anyhow::Error -> UiDbError automatically via From trait
        Ok(self.send_request(UiDbRequest::GetChatHeaders { reply: tx }, rx)?)
    }

    pub fn get_chat_header(&self, topic_id: String) -> Result<UiChatHeader, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatHeader { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_members(&self, topic_id: String) -> Result<Vec<UiContact>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatMembers { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_messages(&self, topic_id: String) -> Result<Vec<UiMessage>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatMessages { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_last_message(&self, topic_id: String) -> Result<Option<UiMessage>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatLastMessage { topic_id, reply: tx }, rx)?)
    }

    pub fn get_chat_data(&self, topic_id: String) -> Result<UiChatData, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetChatData { topic_id, reply: tx }, rx)?)
    }

    pub fn get_contacts(&self) -> Result<Vec<UiContact>, UiDbError> {
        let (tx, rx) = oneshot::channel();
        Ok(self.send_request(UiDbRequest::GetContacts { reply: tx }, rx)?)
    }
}

impl UiDbManagerUniffiObject {
    fn parse_path(path_str: &str) -> Result<PathBuf, UiDbError> {
        PathBuf::from_str(path_str).map_err(|e| UiDbError::InternalError {
            msg: format!("Invalid path: {}", e),
        })
    }
}

#[uniffi::export]
impl UiDbManagerUniffiObject {
    
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Arc<Self>, UiDbError> {
        let db_path = Self::parse_path(&db_path_string)?;
        let inner = UiDbManager::spawn(db_path)?;
        
        Ok(Arc::new(Self { inner }))
    }

    pub fn get_client(&self) -> Arc<UiDbClient> {
        let worker_tx = self.inner.worker_tx();
        
        Arc::new(UiDbClient {
            worker_tx
        })
    }
}