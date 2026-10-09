use anyhow::Result;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};

use super::{ControlRequest, ControlResponse};
use crate::network::chat::ChatSessionsHandle;

#[derive(Clone, Debug)]
pub(in crate::network) struct ControlProtocol {
    pub(in crate::network) chat_sessions: ChatSessionsHandle,
}

impl ProtocolHandler for ControlProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let (mut send, mut recv) = connection.accept_bi().await?;

        let bytes = recv
            .read_to_end(1024 * 1024)
            .await
            .map_err(AcceptError::from_err)?;
        let request: ControlRequest =
            postcard::from_bytes(&bytes).map_err(AcceptError::from_err)?;

        match request {
            ControlRequest::ChatInvite { chat } => {
                let topic_id = chat.topic_id;

                log::debug!(
                    "[CONTROL] received chat invite: topic={}, members={}",
                    topic_id,
                    chat.members.len()
                );

                self.chat_sessions
                    .request_add_chat(chat)
                    .await
                    .map_err(|e| AcceptError::from_err(std::io::Error::other(e.to_string())))?;

                log::info!("[CONTROL] chat invite accepted and session added: topic={topic_id}");

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
