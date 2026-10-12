use anyhow::Result;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use tracing::Instrument;

use super::{ControlRequest, ControlResponse};
use crate::network::chat::ChatSessionsHandle;
use crate::network::short_topic_id;

#[derive(Clone, Debug)]
pub(in crate::network) struct ControlProtocol {
    pub(in crate::network) chat_sessions: ChatSessionsHandle,
    pub(in crate::network) span: tracing::Span,
}

impl ProtocolHandler for ControlProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let span = self.span.clone();
        async {
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

                    tracing::debug!(topic_id = %short_topic_id(&topic_id), members = chat.members.len(), "received chat invite");

                    self.chat_sessions
                        .request_add_chat(chat)
                        .await
                        .map_err(|e| AcceptError::from_err(std::io::Error::other(e.to_string())))?;

                    tracing::info!(topic_id = %short_topic_id(&topic_id), "chat invite accepted and session added");

                    let response = ControlResponse::ChatInviteAccepted { topic_id };

                    let bytes = postcard::to_stdvec(&response).map_err(AcceptError::from_err)?;

                    send.write_all(&bytes)
                        .await
                        .map_err(AcceptError::from_err)?;

                    send.finish().map_err(AcceptError::from_err)?;

                    let _ = tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        connection.closed(),
                    )
                    .await;
                }
            }

            Ok(())
        }
        .instrument(span)
        .await
    }
}
