use tokio::{sync::{mpsc}};
use anyhow::Result;
use tokio::sync::oneshot;
use std::{sync::{Arc, Mutex}, thread::JoinHandle};

use crate::db::{DbManager};
use crate::notifications::UiEventListener;

mod db_client;
mod db_manager;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NwMessage {
    pub topic_id: Vec<u8>,
    pub from_me: bool,
    pub endpoint_id: Option<Vec<u8>>,
    pub content: String,
    pub sent_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NwContact {
    pub name: String,
    pub endpoint_id: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NwChat {
    pub name: Option<String>,
    pub members: Vec<NwContact>,
    pub topic_id: Vec<u8>,
}

pub async fn network_engine(
    db_tx: mpsc::Sender<NwDbRequest>,
    ) -> Result<()>
    {
        todo!()
    }


pub enum NwDbRequest {
    AddMessage {
        message: NwMessage,
        reply: oneshot::Sender<Result<()>>,
    },
    AddContact {
        contact: NwContact,
        reply: oneshot::Sender<Result<()>>,
    },
    AddChat {
        chat: NwChat,
        reply: oneshot::Sender<Result<()>>,
    },
    GetChats {
        reply: oneshot::Sender<Result<Vec<NwChat>>>,
    },
    GetChatMembers {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<NwContact>>>,
    },
    GetChat {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<NwChat>>,
    },
    GetChatMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<NwMessage>>>,
    },
    GetMessages {
        reply: oneshot::Sender<Result<Vec<NwMessage>>>,
    },
}

pub struct NwDbClient {
    worker_tx: mpsc::Sender<NwDbRequest>,
    ui_listener: Arc<dyn UiEventListener>
}

///Network Database Client
/// 
/// 
pub type NwDbManager = DbManager<NwDbRequest>;

//#[cfg(test)]
//#[path = "nw/tests.rs"]
//mod tests;

