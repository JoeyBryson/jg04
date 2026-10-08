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
}

impl ChatInviteActor {
    async fn deliver_pending_invites(&self, store: &InviteStore) -> Result<()> {
        for invite in store.pending_invites().await? {
            match send_chat_invite(&self.router, &invite).await {
                Ok(()) => {
                    store
                        .mark_joined(invite.chat.topic_id, invite.contact.endpoint_id)
                        .await?;
                }

                Err(error) => {
                    log::debug!(
                        "chat invite to {} failed: {error}",
                        invite.contact.endpoint_id
                    );
                }
            }
        }

        Ok(())
    }
}

async fn send_chat_invite(
    router: &Router,
    invite: &PendingChatInvite,
) -> Result<()> {
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