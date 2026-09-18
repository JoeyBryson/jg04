use std::collections::HashMap;

use tokio::sync::mpsc;

use super::super::NwChat;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;

use super::ChatSession;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use iroh::SecretKey;

#[derive(Clone, Debug)]
pub struct ChatSessionManager {
    sender: mpsc::Sender<ChatSessionCommand>,
}
enum ChatSessionCommand {
    Add(NwChat),
    SendMessage { topic_id: TopicId, content: String },
}

impl ChatSessionManager {
    pub fn spawn(db_client: DbClient, gossip: Gossip, secret_key: SecretKey) -> Self {
        let (sender, mut receiver) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut sessions = HashMap::<TopicId, ChatSession>::new();

            while let Some(command) = receiver.recv().await {
                match command {
                    ChatSessionCommand::Add(chat) => {
                        let topic_id = chat.topic_id;

                        match ChatSession::spawn(
                            chat,
                            db_client.clone(),
                            &gossip,
                            secret_key.clone(),
                        )
                        .await
                        {
                            Ok(session) => {
                                sessions.insert(topic_id, session);
                            }
                            Err(error) => {
                                log::error!("failed to spawn chat session: {error}");
                            }
                        }
                    }

                    ChatSessionCommand::SendMessage { topic_id, content } => {
                        let Some(session) = sessions.get(&topic_id) else {
                            log::error!("no active chat session for topic ID: {topic_id}");
                            continue;
                        };

                        if let Err(error) = session.send(content).await {
                            log::error!("failed to send message: {error}");
                        }
                    }
                }
            }
        });

        Self { sender }
    }

    pub fn add_chat(&self, chat: NwChat) -> Result<(), FfiError> {
        self.sender
            .try_send(ChatSessionCommand::Add(chat))
            .map_err(|_| FfiError::internal("failed to send add-chat command"))?;

        Ok(())
    }

    pub fn send_message(&self, topic_id: TopicId, content: String) -> Result<(), FfiError> {
        self.sender
            .try_send(ChatSessionCommand::SendMessage { topic_id, content })
            .map_err(|_| FfiError::internal("failed to send send-message command"))?;

        Ok(())
    }
}
