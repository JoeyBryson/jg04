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
    sender: mpsc::Sender<ChatCommand>,
}

enum ChatCommand {
    Add(NwChat),
}

impl ChatSessionManager {
    fn spawn(
        db_client: DbClient,
        gossip: Gossip,
        secret_key: SecretKey,
    ) -> Self {
        let (sender, mut receiver) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut connectors = HashMap::new();

            while let Some(command) = receiver.recv().await {
                match command {
                    ChatCommand::Add(chat) => {
                        let topic_id = chat.topic_id;

                        match ChatSession::spawn(
                            chat,
                            db_client.clone(),
                            &gossip,
                            secret_key.clone(),
                        )
                        .await
                        {
                            Ok(connector) => {
                                connectors.insert(topic_id, connector);
                            }
                            Err(error) => {
                                log::error!(
                                    "failed to spawn chat connector: {error}"
                                );
                            }
                        }
                    }
                }
            }
        });

        Self { sender }
    }

    fn add_chat(&self, chat: NwChat) -> Result<(), FfiError> {
        self.sender
            .try_send(ChatCommand::Add(chat))
            .map_err(anyhow::Error::from)?;

        Ok(())
    }
}