use crate::db;
use crate::nw;
use tokio::{sync::{mpsc}};
use std::result::Result;
use std::thread::JoinHandle;

use tokio::sync::oneshot;
use uniffi;
use std::any::Any;
use std::sync::{Arc, Mutex};
use anyhow;

#[derive(uniffi::Error, Debug)]
pub enum DbEntrypointError {
    InternalError { msg: String },
}

impl std::fmt::Display for DbEntrypointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbEntrypointError::InternalError { msg } => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DbEntrypointError {}

//experimental datatypes

// #[derive(uniffi::Record)]
// pub struct Contact {
//     pub name: String,
//     pub endpoint_id: Vec<u8>,
// }

// #[derive(uniffi::Record)]
// pub struct Chat {
//     pub name: Option<String>,
//     pub members: Vec<Contact>,
//     pub topic_id: Vec<u8>
// }

// #[derive(uniffi::Record)]
// pub struct Message {
//     pub from_me: bool,
//     pub contact: Option<Contact>,
//     pub content: String,
//     pub sent_at: i64,
// }

// #[derive(uniffi::Record)]
// pub struct ChatWithMessages {
//     pub chat: Chat,
//     pub messages: Vec<Message>
// }

//let's use nw datatypes for now and see how that goes

#[derive(uniffi::Object)]
pub struct DbEntrypoint {
    db_tx: mpsc::Sender<db::Command>,
    join_handle: JoinHandle<()>,
    worker_status: Arc<Mutex<Option<anyhow::Result<()>>>>
}

pub fn create_db_entrypoint(
    db_tx: mpsc::Sender<db::Command>, 
    join_handle: JoinHandle<()>,
    worker_status: Arc<Mutex<Option<anyhow::Result<()>>>>   
) -> DbEntrypoint {
    DbEntrypoint { db_tx, join_handle, worker_status}
}

impl DbEntrypoint {
    fn request<T, F>(&self, f: F) -> Result<T, DbEntrypointError>
    where
        F: FnOnce(oneshot::Sender<anyhow::Result<T>>) -> db::Command,
    {
        if let Some(result) = &*self.worker_status.lock().unwrap() {
            let msg = match result {
                Ok(()) => "DB worker exited".to_string(),
                Err(e) => format!("DB worker crashed: {}", e),
            };

            return Err(DbEntrypointError::InternalError {
                msg: format!("UI001 {}", msg),
            });
        }

        let (tx, rx) = oneshot::channel();

        self.db_tx.blocking_send(f(tx))
            .map_err(|e| DbEntrypointError::InternalError { msg: format!("UI002 send error: {}", e) })?;

        let res = rx.blocking_recv()
            .map_err(|e| DbEntrypointError::InternalError { msg: format!("UI003 recv error: {}", e) })?
            .map_err(|e| DbEntrypointError::InternalError { msg: format!("UI004 DB error: {}", e) })?;
        Ok(res)
    }
}

#[uniffi::export]
impl DbEntrypoint {

    pub fn add_message(&self, message: nw::Message) -> Result<(), DbEntrypointError> {
        self.request(|tx| db::Command::AddMessage { message, reply: tx })
    }

    pub fn add_contact(&self, contact: nw::Contact) -> Result<(), DbEntrypointError> {
        self.request(|tx| db::Command::AddContact { contact, reply: tx })
    }

    pub fn add_chat(&self, chat: nw::Chat) -> Result<(), DbEntrypointError> {
        self.request(|tx| db::Command::AddChat { chat, reply: tx })
    }

    pub fn get_chats(&self) -> Result<Vec<nw::Chat>, DbEntrypointError> {
        self.request(|tx| db::Command::GetChats { reply: tx })
    }

    pub fn get_chat_members(&self, topic_id: Vec<u8>) -> Result<Vec<nw::Contact>, DbEntrypointError> {
        self.request(|tx| db::Command::GetChatMembers { topic_id, reply: tx })
    }

    pub fn get_chat(&self, topic_id: Vec<u8>) -> Result<nw::Chat, DbEntrypointError> {
        self.request(|tx| db::Command::GetChat { topic_id, reply: tx })
    }

    pub fn get_chat_messages(&self, topic_id: Vec<u8>) -> Result<Vec<nw::Message>, DbEntrypointError> {
        self.request(|tx| db::Command::GetChatMessages { topic_id, reply: tx })
    }

    pub fn get_messages(&self) -> Result<Vec<nw::Message>, DbEntrypointError> {
        self.request(|tx| db::Command::GetMessages { reply: tx })
    }
}


