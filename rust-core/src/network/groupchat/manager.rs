use std::collections::HashMap;

use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use iroh::endpoint::{presets, Connection};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::Endpoint;
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Handle, Runtime};
use super::super::{CONTROL_ALPN, 
    NwContact, NwProfile, ControlProtocol, ControlMessage, NwChat};

use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{};
use crate::ui::UiContact;
use crate::ui::UiChatHeader;
use futures_lite::StreamExt;
use iroh::SecretKey;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender},
};
use iroh_gossip::api::Event as GossipEvent;
use super::super::{signed_message::{verify_and_decode, sign_and_encode}};
use super::ChatSession;
use std::time::SystemTime;
use tokio::task::JoinHandle;

pub struct ChatSessionManager {
    sender: mpsc::Sender<ChatSessionCommand>,
}
enum ChatSessionCommand {
    Add(NwChat),
    SendMessage {
        topic_id: TopicId,
        content: String,
    },
}


impl ChatSessionManager {
    pub fn spawn(
        db_client: DbClient,
        gossip: Gossip,
        secret_key: SecretKey,
    ) -> Self {
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
                                log::error!(
                                    "failed to spawn chat session: {error}"
                                );
                            }
                        }
                    }

                    ChatSessionCommand::SendMessage {
                        topic_id,
                        content,
                    } => {
                        let Some(session) = sessions.get(&topic_id) else {
                            log::error!(
                                "no active chat session for topic ID: {topic_id}"
                            );
                            continue;
                        };

                        if let Err(error) = session.send(content).await {
                            log::error!(
                                "failed to send message: {error}"
                            );
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
            .map_err(|_| {
                FfiError::internal(
                    "failed to send add-chat command",
                )
            })?;

        Ok(())
    }

    pub fn send_message(
        &self,
        topic_id: TopicId,
        content: String,
    ) -> Result<(), FfiError> {
        self.sender
            .try_send(ChatSessionCommand::SendMessage {
                topic_id,
                content,
            })
            .map_err(|_| {
                FfiError::internal(
                    "failed to send send-message command",
                )
            })?;

        Ok(())
    }
}