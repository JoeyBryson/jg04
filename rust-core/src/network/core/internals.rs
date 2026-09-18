use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use iroh::protocol::Router;
use iroh::{Endpoint};
use iroh_gossip::net::Gossip;
use tokio::runtime::Handle;

use std::collections::HashMap;
use iroh::endpoint::{presets, Connection};
use super::{NwChat, NwCore};
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{};
use crate::ui::UiContact;
use crate::ui::UiChatHeader;


use super::super::{
    ChatSessionManager,
    CONTROL_ALPN,
    ControlMessage,
    ControlProtocol,
    NwContact,
    NwProfile,
};

#[derive(Debug, thiserror::Error)]
enum SendChatInviteError {
    #[error("timed out waiting for chat invite acceptance")]
    Timeout,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl NwCore {
    pub async fn spawn_base(
        db_client: DbClient,
        profile: NwProfile,
    ) -> Result<Self, FfiError> {
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(profile.secret_key.clone())
            .alpns(vec![
                CONTROL_ALPN.to_vec(),
                iroh_gossip::ALPN.to_vec(),
            ])
            .bind()
            .await
            .map_err(anyhow::Error::from)
            .with_context(|| "endpoint failed to bind")?;

        let control_protocol = ControlProtocol {
            profile: profile.clone(),
            db_client: db_client.clone(),
        };

        let gossip = Gossip::builder().spawn(endpoint.clone());

        let router = Router::builder(endpoint)
            .accept(CONTROL_ALPN, control_protocol)
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let chat_session_manager = ChatSessionManager::spawn(
            db_client.clone(),
            gossip.clone(),
            profile.secret_key.clone(),
        );

        Ok(NwCore {
            runtime_handle: Handle::current(),
            db_client,
            gossip,
            router,
            profile,
            chat_session_manager,
        })
    }

    pub async fn invite_chat_members(
        router: Router,
        chat: NwChat,
    ) -> anyhow::Result<()> {
        for contact in &chat.members {
            let deadline =
                tokio::time::Instant::now() + Duration::from_secs(60);

            loop {
                match Self::send_chat_invite(
                    router.clone(),
                    contact.clone(),
                    chat.clone(),
                    5,
                )
                .await
                {
                    Ok(()) => break,

                    Err(error)
                        if tokio::time::Instant::now() < deadline =>
                    {
                        log::debug!(
                            "chat invite to {} failed: {}, retrying",
                            contact.endpoint_id,
                            error
                        );

                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }

                    Err(error) => {
                        return Err(anyhow::Error::from(error));
                    }
                }
            }
        }

        Ok(())
    }

    async fn send_chat_invite(
        router: Router,
        contact: NwContact,
        chat: NwChat,
        timeout: u64,
    ) -> Result<(), SendChatInviteError> {
        tokio::time::timeout(Duration::from_secs(timeout), async {
            let connection = router
                .endpoint()
                .connect(contact.endpoint_id, CONTROL_ALPN)
                .await?;

            let message = ControlMessage::ChatInvite {
                chat: chat.clone(),
            };

            let bytes = postcard::to_stdvec(&message)?;

            let (mut send, mut recv) = connection.open_bi().await?;

            send.write_all(&bytes).await?;
            send.finish()?;

            let bytes = recv
                .read_to_end(1024 * 1024)
                .await?;

            let response: ControlMessage =
                postcard::from_bytes(&bytes)?;

            match response {
                ControlMessage::ChatInviteAccepted { topic_id }
                    if topic_id == chat.topic_id =>
                {
                    Ok(())
                }

                ControlMessage::ChatInviteAccepted { .. } => {
                    anyhow::bail!(
                        "chat invite accepted for unexpected topic"
                    );
                }

                _ => {
                    anyhow::bail!("unexpected chat invite response");
                }
            }
        })
        .await
        .map_err(|_| SendChatInviteError::Timeout)?
        .map_err(SendChatInviteError::Other)
    }
}