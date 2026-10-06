use std::collections::HashMap;

use anyhow::{anyhow, Result};
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use tokio::sync::{mpsc, oneshot};

use super::super::{NwChat, NwChatMember};
use super::ChatSession;
use crate::network::stores::SessionManagerStore;
use crate::network::NwChatMemberStatus;
use crate::network::NwProfile;
use crate::network::signed_message::MessageData;

#[derive(Clone, Debug)]
pub struct ChatSessionManager {
    sender: mpsc::Sender<ChatSessionCommand>,
}

enum ChatSessionCommand {
    Add {
        chat: NwChat,
        reply: oneshot::Sender<Result<()>>,
    },
    Create {
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

impl ChatSessionManager {
    pub async fn spawn(
        gossip: Gossip,
        profile: NwProfile,
        stores: SessionManagerStore,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);

        let mut sessions = HashMap::new();
        for chat in stores.chats().await? {
            Self::add_chat_session(&mut sessions, chat, &stores, &gossip, &profile).await?;
        }

        tokio::spawn(Self::command_loop(
            receiver,
            sessions,
            gossip,
            profile,
            stores,
        ));

        Ok(Self { sender })
    }

    async fn command_loop(
        mut receiver: mpsc::Receiver<ChatSessionCommand>,
        mut sessions: HashMap<TopicId, ChatSession>,
        gossip: Gossip,
        profile: NwProfile,
        stores: SessionManagerStore,
    ) {
        while let Some(command) = receiver.recv().await {
            match command {
                ChatSessionCommand::Add { chat, reply } => {
                    let result = Self::persist_and_activate(
                        &mut sessions,
                        chat,
                        &stores,
                        &gossip,
                        &profile,
                    )
                    .await;

                    let _ = reply.send(result);
                }

                ChatSessionCommand::Create {
                    mut members,
                    name,
                    reply,
                } => {
                    members.push(NwChatMember {
                        contact: profile.contact.clone(),
                        status: NwChatMemberStatus::Joined,
                    });

                    let topic_id = TopicId::from_bytes(rand::random());
                    let chat = NwChat {
                        name,
                        members,
                        topic_id,
                    };

                    let result = Self::persist_and_activate(
                        &mut sessions,
                        chat,
                        &stores,
                        &gossip,
                        &profile,
                    )
                    .await
                    .map(|()| topic_id);

                    let _ = reply.send(result);
                }

                ChatSessionCommand::SendMessage {
                    topic_id,
                    message_data,
                    reply,
                } => {
                    let result = match sessions.get(&topic_id) {
                        Some(session) => session.send(message_data).await,
                        None => Err(anyhow!(
                            "no active chat session for topic ID: {topic_id}"
                        )),
                    };

                    let _ = reply.send(result);
                }
            }
        }
    }

    async fn persist_and_activate(
        sessions: &mut HashMap<TopicId, ChatSession>,
        chat: NwChat,
        stores: &SessionManagerStore,
        gossip: &Gossip,
        profile: &NwProfile,
    ) -> Result<()> {
        if sessions.contains_key(&chat.topic_id) {
            return Err(anyhow!("chat already in sessions"));
        }

        stores.save_chat(chat.clone()).await?;

        Self::add_chat_session(sessions, chat, stores, gossip, profile).await
    }

    async fn add_chat_session(
        sessions: &mut HashMap<TopicId, ChatSession>,
        chat: NwChat,
        stores: &SessionManagerStore,
        gossip: &Gossip,
        profile: &NwProfile,
    ) -> Result<()> {
        let topic_id = chat.topic_id;

        if sessions.contains_key(&topic_id) {
            return Err(anyhow!("chat already in sessions"));
        }

        let session = ChatSession::spawn(
            chat,
            stores.spawn_session_store(topic_id),
            gossip,
            profile.clone(),
        )
        .await?;

        sessions.insert(topic_id, session);

        Ok(())
    }

    pub async fn add_chat(&self, chat: NwChat) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::Add { chat, reply })
            .await?;

        receiver.await?
    }

    pub async fn create_chat(
        &self,
        members: Vec<NwChatMember>,
        name: Option<String>,
    ) -> Result<TopicId> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::Create { members, name, reply })
            .await?;

        receiver.await?
    }

    pub async fn send_message(
        &self,
        topic_id: TopicId,
        message_data: MessageData
    ) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatSessionCommand::SendMessage {
                topic_id,
                message_data,
                reply,
            })
            .await?;

        receiver.await?
    }
}