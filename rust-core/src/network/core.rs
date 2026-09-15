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

use super::chat_connector::NwChatConnector;
use super::NwChat;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{NwContact, NwProfile};
use crate::ui::UiChatHeader;

#[derive(uniffi::Object)]
pub struct NwCore {
    runtime_handle: Handle,
    db_client: DbClient,
    gossip: Gossip,
    router: Router,
    profile: NwProfile,
    chat_connectors: HashMap<TopicId, NwChatConnector>,
}

#[uniffi::export]
impl NwCore {
    #[uniffi::constructor]
    pub fn spawn(db_client: Arc<DbClient>) -> Result<Arc<Self>, FfiError> {
        log::info!("starting nw_core::spawn");

        let db_client = Arc::unwrap_or_clone(db_client);

        let profile = db_client
            .get_nw_profile()
            .map_err(anyhow::Error::from)
            .with_context(|| "invalid private key")?;

        let runtime = Runtime::new()
            .map_err(anyhow::Error::from)?;

        let runtime_handle = runtime.handle().clone();

        let (gossip, router, chat_connectors) = runtime_handle.block_on(async {
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

            let chats = db_client.get_nw_chats().await?;
            let mut chat_connectors = HashMap::with_capacity(chats.len());

            for chat in chats {
                let topic_id = chat.topic_id;

                let connector = NwChatConnector::spawn(
                    chat,
                    db_client.clone(),
                    &gossip,
                    profile.secret_key.clone(),
                )
                .await
                .with_context(|| "chat connector failed to spawn")?;

                chat_connectors.insert(topic_id, connector);
            }

            Ok::<_, FfiError>((gossip, router, chat_connectors))
        })?;

        std::thread::spawn(move || {
            runtime.block_on(std::future::pending::<()>());
        });

        log::info!("nw_core::spawn completed");

        Ok(Arc::new(NwCore {
            runtime_handle,
            db_client,
            gossip,
            router,
            profile,
            chat_connectors,
        }))
    }

    fn send_message(
        &self,
        content: String,
        chat_id: String,
    ) -> Result<(), FfiError> {
        let mut topic_id_bytes = [0u8; 32];

        hex::decode_to_slice(&chat_id, &mut topic_id_bytes)
            .map_err(anyhow::Error::from)?;

        let topic_id = TopicId::from_bytes(topic_id_bytes);

        let connector = self
            .chat_connectors
            .get(&topic_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No active connector for topic ID: {}",
                    chat_id
                )
            })?;

        self.runtime_handle
            .block_on(connector.send(content))
            .map_err(anyhow::Error::from)?;

        Ok(())
    }

    fn invite_chat_members(
        &self,
        chat_header: UiChatHeader,
    ) -> Result<(), FfiError> {

        let contacts: Vec<NwContact> = chat_header
            .members
            .iter()
            .cloned()
            .map(Into::into)
            .collect();

        let chat: NwChat = chat_header.into();

        self.runtime_handle.block_on(async {
            for contact in contacts {
                let deadline =
                    tokio::time::Instant::now() + Duration::from_secs(60);

                loop {
                    match self
                        .send_chat_invite(
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

            Ok::<_, anyhow::Error>(())
        })?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum SendChatInviteError {
    #[error("timed out waiting for chat invite acceptance")]
    Timeout,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl NwCore {
    async fn send_chat_invite(
        &self,
        contact: NwContact,
        chat: NwChat,
        timeout: u64,
    ) -> Result<(), SendChatInviteError> {
        tokio::time::timeout(Duration::from_secs(timeout), async {
            let connection = self
                .router
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

#[derive(Debug, Serialize, Deserialize)]
enum ControlMessage {
    ChatInvite {
        chat: NwChat,
    },
    ChatInviteAccepted {
        topic_id: TopicId,
    },
}

const CONTROL_ALPN: &[u8] = b"iroh-example/echo/0";

#[derive(Debug, Clone)]
struct ControlProtocol {
    profile: NwProfile,
    db_client: DbClient,
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