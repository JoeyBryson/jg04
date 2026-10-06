use std::{collections::HashMap, time::Duration};

use anyhow::{Context, Result};
use iroh::protocol::Router;
use iroh_gossip::proto::TopicId;
use tokio::sync::{mpsc, oneshot};

use super::{
    ControlRequest, ControlResponse, NwChat, NwChatMemberStatus, NwContact, CONTROL_ALPN,
};
use crate::network::stores::InviteStore;

#[derive(Clone, Debug)]
pub struct ChatInviteManager {
    sender: mpsc::Sender<ChatInviteCommand>,
}

struct ChatInviteActor {
    router: Router,
    pending: HashMap<(TopicId, iroh::EndpointId), PendingInvite>,
}

enum ChatInviteCommand {
    Refresh {
        reply: oneshot::Sender<Result<()>>,
    },
}

#[derive(Clone, Debug)]
struct PendingInvite {
    chat: NwChat,
    contact: NwContact,
}

impl ChatInviteManager {
    pub(in crate::network) async fn spawn(router: Router, store: InviteStore) -> Result<Self> {
        let (sender, mut receiver) = mpsc::channel(8);

        tokio::spawn(async move {
            let mut actor = ChatInviteActor {
                router,
                pending: HashMap::new(),
            };

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
                            ChatInviteCommand::Refresh { reply } => {
                                let _ = reply.send(actor.run_cycle(&store).await);
                            }
                        }
                    }

                    _ = interval.tick() => {
                        if let Err(error) = actor.run_cycle(&store).await {
                            log::warn!("chat invite cycle failed: {error}");
                        }
                    }
                }
            }
        });

        let manager = Self { sender };

        manager.refresh().await?;

        Ok(manager)
    }

    pub async fn refresh(&self) -> Result<()> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(ChatInviteCommand::Refresh { reply })
            .await?;

        receiver
            .await
            .context("chat invite actor stopped")?
    }
}

impl ChatInviteActor {
    async fn run_cycle(&mut self, store: &InviteStore) -> Result<()> {
        self.reconcile(store).await?;

        let pending = self.pending.values().cloned().collect::<Vec<_>>();

        for invite in pending {
            match send_chat_invite(&self.router, &invite).await {
                Ok(()) => {
                    store
                        .mark_joined(invite.chat.topic_id, invite.contact.endpoint_id)
                        .await?;

                    self.pending
                        .remove(&(invite.chat.topic_id, invite.contact.endpoint_id));
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

    async fn reconcile(&mut self, store: &InviteStore) -> Result<()> {
        let chats = store.chats().await?;
        let mut pending = HashMap::new();

        for chat in chats {
            for member in chat
                .members
                .iter()
                .filter(|member| member.status != NwChatMemberStatus::Joined)
            {
                pending.insert(
                    (chat.topic_id, member.contact.endpoint_id),
                    PendingInvite {
                        chat: chat.clone(),
                        contact: member.contact.clone(),
                    },
                );
            }
        }

        self.pending = pending;

        Ok(())
    }
}

async fn send_chat_invite(
    router: &Router,
    invite: &PendingInvite,
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