use std::collections::HashMap;

use anyhow::Result;
use iroh::SecretKey;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use tokio::sync::{mpsc, oneshot};

use super::super::NwChat;
use super::ChatSession;
use crate::database::client::DbClient;

#[derive(Clone, Debug)]
pub struct ChatSessionManager {
    sender: mpsc::Sender<ChatSessionCommand>,
}

enum ChatSessionCommand {
    Add {
        chat: NwChat,
        reply: oneshot::Sender<Result<()>>,
    },
    SendMessage {
        topic_id: TopicId,
        content: String,
        reply: oneshot::Sender<Result<()>>,
    },
}

impl ChatSessionManager {
    pub async fn spawn(db_client: DbClient, gossip: Gossip, secret_key: SecretKey) -> Result<Self> {
        let (sender, mut receiver) = mpsc::channel(32);
        let db_client_clone = db_client.clone();

        tokio::spawn(async move {
            let mut sessions = HashMap::<TopicId, ChatSession>::new();

            while let Some(command) = receiver.recv().await {
                match command {
                    ChatSessionCommand::Add { chat, reply } => {
                        let topic_id = chat.topic_id;

                        if sessions.contains_key(&topic_id) {
                            let _ = reply.send(Ok(()));
                            continue;
                        }

                        let result = (async || {
                            ChatSession::spawn(
                                chat.clone(),
                                db_client.clone(),
                                &gossip,
                                secret_key.clone(),
                            )
                            .await
                            .map(|session| {
                                sessions.insert(topic_id, session);
                            })?;

                            db_client.add_nw_chat_sync(chat)
                        })()
                        .await;

                        let _ = reply.send(result);
                    }

                    ChatSessionCommand::SendMessage {
                        topic_id,
                        content,
                        reply,
                    } => {
                        let result = match sessions.get(&topic_id) {
                            Some(session) => session.send(content).await,
                            None => Err(anyhow::anyhow!(
                                "no active chat session for topic ID: {topic_id}"
                            )),
                        };

                        let _ = reply.send(result);
                    }
                }
            }
        });

        let chat_session_manager = Self{ sender };

        let chats = db_client_clone.get_nw_chats_sync()?;

        for chat in chats {
            chat_session_manager.add_chat(chat)?;
        }

        Ok(chat_session_manager)
    }

    pub fn add_chat(&self, chat: NwChat) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .blocking_send(ChatSessionCommand::Add { chat, reply })?;

        receiver.blocking_recv()?
    }

    pub async fn add_chat_async(&self, chat: NwChat) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::Add { chat, reply })
            .await?;

        receiver.await?
    }

    pub fn send_message(&self, topic_id: TopicId, content: String) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender.blocking_send(ChatSessionCommand::SendMessage {
            topic_id,
            content,
            reply,
        })?;

        receiver.blocking_recv()?
    }
}
