use std::time::Duration;

use anyhow::{Context, Result};
use iroh::protocol::Router;
use tokio::sync::{mpsc, oneshot};
use tracing::Instrument;

use super::{ControlRequest, ControlResponse};
use crate::network::persistence::{InviteStore, PendingChatInvite};
use crate::network::{CONTROL_ALPN, short_topic_id};

#[derive(Clone, Debug)]
/// Cloneable command handle for the actor that delivers pending chat invites.
pub struct ChatInvitesHandle {
    sender: mpsc::Sender<ChatInviteCommand>,
}

struct ChatInviteActor {
    router: Router,
}

enum ChatInviteCommand {
    DeliverPendingInvites { reply: oneshot::Sender<Result<()>> },
    Shutdown { reply: oneshot::Sender<Result<()>> },
}

impl ChatInvitesHandle {
    pub(in crate::network) async fn spawn(router: Router, store: InviteStore) -> Result<Self> {
        let (sender, mut receiver) = mpsc::channel(8);

        tokio::spawn(
            async move {
                let actor = ChatInviteActor { router };

                let mut interval = tokio::time::interval_at(
                    tokio::time::Instant::now() + Duration::from_secs(5),
                    Duration::from_secs(5),
                );

                loop {
                    tokio::select! {
                        command = receiver.recv() => {
                            let Some(command) = command else {
                                break;
                            };

                            match command {
                                ChatInviteCommand::DeliverPendingInvites { reply } => {
                                    let _ = reply.send(actor.deliver_pending_invites(&store).await);
                                }
                                ChatInviteCommand::Shutdown { reply } => {
                                    let _ = reply.send(actor.shutdown().await);
                                    break;
                                }
                            }
                        }

                        _ = interval.tick() => {
                            if let Err(error) = actor.deliver_pending_invites(&store).await {
                                    tracing::warn!(error = %error, "invite delivery cycle failed");
                            }
                        }
                    }
                }
            }
            .instrument(tracing::Span::current()),
        );

        let manager = Self { sender };

        manager.request_invite_delivery().await?;

        Ok(manager)
    }

    pub async fn request_invite_delivery(&self) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatInviteCommand::DeliverPendingInvites { reply })
            .await?;

        receiver.await.context("chat invite actor stopped")?
    }

    pub async fn shutdown(&self) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatInviteCommand::Shutdown { reply })
            .await
            .context("chat invite actor stopped before shutdown request")?;

        receiver
            .await
            .context("chat invite actor stopped during shutdown")?
    }
}

impl ChatInviteActor {
    async fn deliver_pending_invites(&self, store: &InviteStore) -> Result<()> {
        let pending_invites = store.pending_invites().await?;
        if pending_invites.is_empty() {
            return Ok(());
        }

        tracing::debug!(count = pending_invites.len(), "delivering pending invites");

        for invite in pending_invites {
            tracing::debug!(topic_id = %short_topic_id(&invite.chat.topic_id), target = %short_endpoint_id(&invite.contact.endpoint_id), "sending chat invite");

            match send_chat_invite(&self.router, &invite).await {
                Ok(()) => {
                    tracing::info!(topic_id = %short_topic_id(&invite.chat.topic_id), target = %short_endpoint_id(&invite.contact.endpoint_id), "invite accepted");

                    store
                        .mark_joined(invite.chat.topic_id, invite.contact.endpoint_id)
                        .await?;

                    tracing::debug!(topic_id = %short_topic_id(&invite.chat.topic_id), member = %short_endpoint_id(&invite.contact.endpoint_id), "marked member joined");
                }

                Err(error) => {
                    let error_text = error.to_string();
                    tracing::warn!(topic_id = %short_topic_id(&invite.chat.topic_id), target = %short_endpoint_id(&invite.contact.endpoint_id), error = %error_text, "chat invite failed; member remains pending until acceptance is acknowledged");
                }
            }
        }

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        self.router.shutdown().await?;
        Ok(())
    }
}

async fn send_chat_invite(router: &Router, invite: &PendingChatInvite) -> Result<()> {
    tracing::trace!(topic_id = %short_topic_id(&invite.chat.topic_id), target = %short_endpoint_id(&invite.contact.endpoint_id), "opening control connection");

    tokio::time::timeout(Duration::from_secs(5), async {
        let connection = router
            .endpoint()
            .connect(invite.contact.endpoint_id, CONTROL_ALPN)
            .await?;

        let message = ControlRequest::ChatInvite {
            chat: invite.chat.clone(),
        };

        let bytes = postcard::to_stdvec(&message)?;

        let (mut send, mut recv) = connection.open_bi().await?;

        send.write_all(&bytes).await?;
        send.finish()?;

        let bytes = recv.read_to_end(1024 * 1024).await?;

        let response: ControlResponse = postcard::from_bytes(&bytes)?;

        let result = match response {
            ControlResponse::ChatInviteAccepted { topic_id }
                if topic_id == invite.chat.topic_id =>
            {
                tracing::trace!(topic_id = %short_topic_id(&topic_id), target = %short_endpoint_id(&invite.contact.endpoint_id), "received invite acceptance");
                Ok(())
            }

            ControlResponse::ChatInviteAccepted { .. } => {
                anyhow::bail!("chat invite accepted for unexpected topic")
            }
        };

        connection.close(0u32.into(), b"invite acknowledgement received");
        result
    })
    .await
    .map_err(|_| anyhow::anyhow!("timed out waiting for chat invite acceptance"))??;

    Ok(())
}

fn short_endpoint_id(endpoint_id: &impl std::fmt::Display) -> String {
    let id = endpoint_id.to_string();
    id[..10].to_owned()
}
