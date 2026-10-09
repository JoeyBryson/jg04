use std::time::Duration;

use anyhow::{Context, Result};
use iroh::protocol::Router;
use tokio::sync::{mpsc, oneshot};

use super::{ControlRequest, ControlResponse};
use crate::network::persistence::{InviteStore, PendingChatInvite};
use crate::network::CONTROL_ALPN;

#[derive(Clone, Debug)]
/// Cloneable command handle for the actor that delivers pending chat invites.
pub struct ChatInvitesHandle {
    sender: mpsc::Sender<ChatInviteCommand>,
}

struct ChatInviteActor {
    router: Router,
}

enum ChatInviteCommand {
    DeliverPendingInvites {
        reply: oneshot::Sender<Result<()>>,
    },
    Shutdown {
        reply: oneshot::Sender<Result<()>>,
    },
}

impl ChatInvitesHandle {
    pub(in crate::network) async fn spawn(router: Router, store: InviteStore) -> Result<Self> {
        let (sender, mut receiver) = mpsc::channel(8);

        tokio::spawn(async move {
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
                            log::warn!("chat invite cycle failed: {error}");
                        }
                    }
                }
            }
        });

        let manager = Self { sender };

        manager.request_invite_delivery().await?;

        Ok(manager)
    }

    pub async fn request_invite_delivery(&self) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatInviteCommand::DeliverPendingInvites { reply })
            .await?;

        receiver
            .await
            .context("chat invite actor stopped")?
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
        log::debug!(
            "[INVITES] delivering pending invites: count={}",
            pending_invites.len()
        );

        for invite in pending_invites {
            log::debug!(
                "[INVITES] sending chat invite: topic={}, target={}",
                invite.chat.topic_id,
                invite.contact.endpoint_id
            );

            match send_chat_invite(&self.router, &invite).await {
                Ok(()) => {
                    log::info!(
                        "[INVITES] invite accepted: topic={}, target={}",
                        invite.chat.topic_id,
                        invite.contact.endpoint_id
                    );

                    store
                        .mark_joined(invite.chat.topic_id, invite.contact.endpoint_id)
                        .await?;

                    log::debug!(
                        "[INVITES] marked member joined: topic={}, member={}",
                        invite.chat.topic_id,
                        invite.contact.endpoint_id
                    );
                }

                Err(error) => {
                    let error_text = error.to_string();
                    let transient_response_race = error_text.contains("read error: connection lost")
                        || error_text.contains("Hit the end of buffer, expected more data");

                    if transient_response_race {
                        log::debug!(
                            "[INVITES] transient invite delivery race: topic={}, target={}, error={}",
                            invite.chat.topic_id,
                            invite.contact.endpoint_id,
                            error_text,
                        );
                    } else {
                        log::warn!(
                            "[INVITES] chat invite failed: topic={}, target={}, error={}",
                            invite.chat.topic_id,
                            invite.contact.endpoint_id,
                            error_text,
                        );
                    }
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

async fn send_chat_invite(
    router: &Router,
    invite: &PendingChatInvite,
) -> Result<()> {
    log::trace!(
        "[INVITES] opening control connection: topic={}, target={}",
        invite.chat.topic_id,
        invite.contact.endpoint_id
    );

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

        match response {
            ControlResponse::ChatInviteAccepted { topic_id }
                if topic_id == invite.chat.topic_id =>
            {
                log::trace!(
                    "[INVITES] received invite acceptance: topic={}, target={}",
                    topic_id,
                    invite.contact.endpoint_id
                );
                Ok(())
            }

            ControlResponse::ChatInviteAccepted { .. } => {
                anyhow::bail!("chat invite accepted for unexpected topic")
            }
        }
    })
    .await
    .map_err(|_| anyhow::anyhow!("timed out waiting for chat invite acceptance"))??;

    Ok(())
}