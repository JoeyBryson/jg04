use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use super::{ChatSessionManager, NwChat, NwChatMemberStatus, NwContact, CONTROL_ALPN};
use crate::database::client::DbClient;
use crate::network::NwProfile;
pub mod chat_invite;

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlRequest {
    ChatInvite { chat: NwChat },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlResponse {
    ChatInviteAccepted { topic_id: TopicId },
}



#[derive(Clone, Debug)]
pub(super) struct ControlProtocol {
    pub(super) profile: NwProfile,
    pub(super) db_client: DbClient,
    pub(super) chat_session_manager: ChatSessionManager,
}

impl ProtocolHandler for ControlProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let sender_id = connection.remote_id();

        let sender = self
            .db_client
            .get_nw_contact(sender_id)
            .await
            .map_err(|e| AcceptError::from_err(std::io::Error::other(e.to_string())))?;

        let (mut send, mut recv) = connection.accept_bi().await?;

        let bytes = recv
            .read_to_end(1024 * 1024)
            .await
            .map_err(AcceptError::from_err)?;
        let request: ControlRequest =
            postcard::from_bytes(&bytes).map_err(AcceptError::from_err)?;

        match request {
            ControlRequest::ChatInvite { mut chat } => {
                for member in &mut chat.members {
                    if member.contact.endpoint_id == self.profile.contact.endpoint_id {
                        member.contact = sender.clone();
                        member.status = NwChatMemberStatus::Joined;
                    }
                }

                let topic_id = chat.topic_id;

                self.db_client
                    .add_nw_chat(chat.clone())
                    .await
                    .map_err(|e| AcceptError::from_err(std::io::Error::other(e.to_string())))?;

                self.chat_session_manager
                    .add_chat_async(chat)
                    .await
                    .map_err(|e| AcceptError::from_err(std::io::Error::other(e.to_string())))?;

                let response = ControlResponse::ChatInviteAccepted { topic_id };

                let bytes = postcard::to_stdvec(&response).map_err(AcceptError::from_err)?;

                send.write_all(&bytes)
                    .await
                    .map_err(AcceptError::from_err)?;

                send.finish().map_err(AcceptError::from_err)?;
            }
        }

        Ok(())
    }
}
