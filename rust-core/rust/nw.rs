use crate::db;
use tokio::{sync::{mpsc}};
use anyhow::Result;
use tokio::sync::oneshot;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Message {
    pub topic_id: Vec<u8>,
    pub from_me: bool,
    pub endpoint_id: Option<Vec<u8>>,
    pub content: String,
    pub sent_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Contact {
    pub name: String,
    pub endpoint_id: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Chat {
    pub name: Option<String>,
    pub members: Vec<Contact>,
    pub topic_id: Vec<u8>,
}

pub async fn network_engine(
    db_tx: mpsc::Sender<db::Command>,
    ) -> Result<()>
    {
        todo!()
    }

pub struct AsyncDbEntrypoint {
    db_tx: mpsc::Sender<db::Command>
}

pub fn create_async_db_entrypoint(
    db_tx: mpsc::Sender<db::Command>
) -> AsyncDbEntrypoint {
    AsyncDbEntrypoint { db_tx }
}

impl AsyncDbEntrypoint {

    async fn request<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> db::Command,
    {
        let (tx, rx) = oneshot::channel();

        self.db_tx.send(f(tx)).await?;

        let res = rx.await??;
        Ok(res)
    }

    pub async fn add_message(&self, message: Message) -> Result<()> {
        self.request(|tx| db::Command::AddMessage {
            message,
            reply: tx,
        }).await
    }

    pub async fn add_contact(&self, contact: Contact) -> Result<()> {
        self.request(|tx| db::Command::AddContact {
            contact,
            reply: tx,
        }).await
    }

    pub async fn add_chat(&self, chat: Chat) -> Result<()> {
        self.request(|tx| db::Command::AddChat {
            chat,
            reply: tx,
        }).await
    }

    pub async fn get_chats(&self) -> Result<Vec<Chat>> {
        self.request(|tx| db::Command::GetChats {
            reply: tx,
        }).await
    }

    pub async fn get_chat_members(&self, topic_id: Vec<u8>) -> Result<Vec<Contact>> {
        self.request(|tx| db::Command::GetChatMembers {
            topic_id,
            reply: tx,
        }).await
    }

    pub async fn get_chat(&self, topic_id: Vec<u8>) -> Result<Chat> {
        self.request(|tx| db::Command::GetChat {
            topic_id,
            reply: tx,
        }).await
    }

    pub async fn get_chat_messages(&self, topic_id: Vec<u8>) -> Result<Vec<Message>> {
        self.request(|tx| db::Command::GetChatMessages {
            topic_id,
            reply: tx,
        }).await
    }

    pub async fn get_messages(&self) -> Result<Vec<Message>> {
        self.request(|tx| db::Command::GetMessages {
            reply: tx,
        }).await
    }

    pub async fn add_sample_chat(&self) -> Result<()> {

    let alice = Contact {
        name: "Alice".to_string(),
        endpoint_id: vec![0u8; 32],
    };

    let bob = Contact {
        name: "Bob".to_string(),
        endpoint_id: vec![1u8; 32],
    };

    self.add_contact(alice.clone()).await?;

    self.add_contact(bob.clone()).await?;

    let topic_id = vec![2u8; 32];

    let chat = Chat {
        name: Some("Sample Chat".to_string()),
        members: vec![alice.clone(), bob.clone()],
        topic_id: topic_id.clone(),
    };

    self.add_chat(chat).await?;
    let mut sent_at = 1640995200i64;

    for i in 0..30 {
        let from_me = i % 3 == 0;

        let endpoint_id = if from_me {
            None
        } else {
            Some(if i % 2 == 0 {
                alice.endpoint_id.clone()
            } else {
                bob.endpoint_id.clone()
            })
        };

        let message = Message {
            topic_id: topic_id.clone(),
            from_me,
            endpoint_id,
            content: format!("Sample message {}", i + 1),
            sent_at,
        };

        self.add_message(message).await?;

        sent_at += 60;
    }

    Ok(())
}
}

#[cfg(test)]
#[path = "nw/tests.rs"]
mod tests;

