
use std::collections::HashMap;
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

use super::groupchat::ChatSession;
use super::NwChat;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{NwContact, NwProfile};
use crate::ui::UiChatHeader;

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlMessage {
    ChatInvite {
        chat: NwChat,
    },
    ChatInviteAccepted {
        topic_id: TopicId,
    },
}

#[derive(Debug, Clone)]
pub(super) struct ControlProtocol {
    pub(super) profile: NwProfile,
    pub(super) db_client: DbClient,
}

impl ProtocolHandler for ControlProtocol {
    async fn accept(
        &self,
        connection: Connection,
    ) -> Result<(), AcceptError> {
        let sender_id = connection.remote_id();

        let sender = self
            .db_client
            .get_nw_contact(sender_id)
            .await
            .map_err(|e| {
                AcceptError::from_err(
                    std::io::Error::other(e.to_string()),
                )
            })?;

        let (mut send, mut recv) = connection.accept_bi().await?;

        let bytes = recv
            .read_to_end(1024 * 1024)
            .await
            .map_err(AcceptError::from_err)?;

        let message: ControlMessage =
            postcard::from_bytes(&bytes)
                .map_err(AcceptError::from_err)?;

        match message {
            ControlMessage::ChatInvite { chat } => {
                let mut chat = chat;

                for member in &mut chat.members {
                    if *member == self.profile.contact {
                        *member = sender.clone();
                    }
                }

                let topic_id = chat.topic_id;

                self.db_client
                    .add_nw_chat(chat)
                    .await
                    .map_err(|e| {
                        AcceptError::from_err(
                            std::io::Error::other(e.to_string()),
                        )
                    })?;

                let response =
                    ControlMessage::ChatInviteAccepted { topic_id };

                let bytes = postcard::to_stdvec(&response)
                    .map_err(AcceptError::from_err)?;

                send.write_all(&bytes)
                    .await
                    .map_err(AcceptError::from_err)?;

                send.finish()
                    .map_err(AcceptError::from_err)?;
            }

            ControlMessage::ChatInviteAccepted { .. } => {}
        }

        Ok(())
    }
}