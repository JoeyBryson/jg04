use std::collections::HashMap;

use anyhow::{anyhow, Result};
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use tokio::sync::{mpsc, oneshot};

use crate::network::NwChat;
use crate::network::NwChatMember;
use super::session::ChatSession;
use crate::network::persistence::SessionManagerStore;
use crate::network::NwChatMemberStatus;
use crate::network::messaging::signed::MessageData;

#[derive(Clone, Debug)]
/// Cloneable command handle for the actor that owns all active chat sessions.
pub struct ChatSessionsHandle {
    sender: mpsc::Sender<ChatSessionCommand>,
}

enum ChatSessionCommand {
    AddChat {
        chat: NwChat,
        reply: oneshot::Sender<Result<()>>,
    },
    CreateChat {
        members: Vec<NwChatMember>,
        name: Option<String>,
        reply: oneshot::Sender<Result<TopicId>>,
    },
    SendMessage {
        topic_id: TopicId,
        message_data: MessageData,
        reply: oneshot::Sender<Result<()>>,
    },
}

impl ChatSessionsHandle {
    pub async fn spawn(
        gossip: Gossip,
        store: SessionManagerStore,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);

        let mut sessions = HashMap::new();
        for chat in store.chats().await? {
            Self::add_chat_session(&mut sessions, chat, &store, &gossip).await?;
        }

        tokio::spawn(Self::command_loop(
            receiver,
            sessions,
            gossip,
            store,
        ));

        Ok(Self { sender })
    }

    pub async fn request_add_chat(&self, chat: NwChat) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::AddChat { chat, reply })
            .await?;

        receiver.await?
    }

    pub async fn request_create_chat(
        &self,
        members: Vec<NwChatMember>,
        name: Option<String>,
    ) -> Result<TopicId> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::CreateChat { members, name, reply })
            .await?;

        receiver.await?
    }

    pub async fn request_send_message(
        &self,
        topic_id: TopicId,
        message_data: MessageData
    ) -> Result<()> {
        log::debug!(
            "[CHAT-SESSIONS] request_send_message: topic={}, sent_at={}, content_len={}",
            topic_id,
            message_data.sent_at,
            message_data.content.len()
        );

        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::SendMessage {
                topic_id,
                message_data,
                reply,
            })
            .await?;

        let result = receiver.await?;

        if let Err(error) = &result {
            log::warn!(
                "[CHAT-SESSIONS] request_send_message failed for topic={}: {}",
                topic_id,
                error
            );
        }

        result
    }

    async fn save_chat_and_start_session(
        sessions: &mut HashMap<TopicId, ChatSession>,
        chat: NwChat,
        store: &SessionManagerStore,
        gossip: &Gossip,
    ) -> Result<()> {
        if sessions.contains_key(&chat.topic_id) {
            log::debug!(
                "[CHAT-SESSIONS] save_chat_and_start_session: topic already active, skipping duplicate add: {}",
                chat.topic_id
            );
            return Ok(());
        }

        store.save_chat(chat.clone()).await?;

        Self::add_chat_session(sessions, chat, store, gossip).await
    }

    async fn add_chat_session(
        sessions: &mut HashMap<TopicId, ChatSession>,
        chat: NwChat,
        store: &SessionManagerStore,
        gossip: &Gossip,
    ) -> Result<()> {
        let topic_id = chat.topic_id;

        if sessions.contains_key(&topic_id) {
            return Err(anyhow!("chat already in sessions"));
        }

        let session = ChatSession::spawn(store.spawn_session_store(chat), gossip).await?;

        sessions.insert(topic_id, session);

        log::info!(
            "[CHAT-SESSIONS] added chat session for topic={}, total_sessions={}",
            topic_id,
            sessions.len()
        );

        Ok(())
    }

    async fn command_loop(
        mut receiver: mpsc::Receiver<ChatSessionCommand>,
        mut sessions: HashMap<TopicId, ChatSession>,
        gossip: Gossip,
        store: SessionManagerStore,
    ) {
        while let Some(command) = receiver.recv().await {
            match command {
                ChatSessionCommand::AddChat { chat, reply } => {
                    let result = Self::save_chat_and_start_session(
                        &mut sessions,
                        chat,
                        &store,
                        &gossip,
                    )
                    .await;

                    let _ = reply.send(result);
                }

                ChatSessionCommand::CreateChat {
                    members,
                    name,
                    reply,
                } => {
                    let result = Self::create_chat_and_start_session(
                        &mut sessions,
                        members,
                        name,
                        &store,
                        &gossip,
                    )
                    .await;

                    let _ = reply.send(result);
                }

                ChatSessionCommand::SendMessage {
                    topic_id,
                    message_data,
                    reply,
                } => {
                    log::debug!(
                        "[CHAT-SESSIONS] command SendMessage: topic={}, sent_at={}, content_len={}, active_sessions={}",
                        topic_id,
                        message_data.sent_at,
                        message_data.content.len(),
                        sessions.len()
                    );

                    let result = match sessions.get(&topic_id) {
                        Some(session) => session.send(message_data).await,
                        None => {
                            let known_topics = sessions
                                .keys()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(",");

                            Err(anyhow!(
                                "no active chat session for topic ID: {topic_id}. known_topics=[{known_topics}]"
                            ))
                        }
                    };

                    if let Err(error) = &result {
                        log::warn!(
                            "[CHAT-SESSIONS] SendMessage failed for topic={}: {}",
                            topic_id,
                            error
                        );
                    }

                    let _ = reply.send(result);
                }
            }
        }
    }

    async fn create_chat_and_start_session(
        sessions: &mut HashMap<TopicId, ChatSession>,
        mut members: Vec<NwChatMember>,
        name: Option<String>,
        store: &SessionManagerStore,
        gossip: &Gossip,
    ) -> Result<TopicId> {
        members.push(NwChatMember {
            contact: store.profile().contact.clone(),
            status: NwChatMemberStatus::Joined,
        });

        let topic_id = TopicId::from_bytes(rand::random());
        let chat = NwChat {
            name,
            members,
            topic_id,
        };

        log::info!(
            "[CHAT-SESSIONS] creating chat session for new topic={}, members={}",
            topic_id,
            chat.members.len()
        );

        Self::save_chat_and_start_session(sessions, chat, store, gossip).await?;
        Ok(topic_id)
    }
}