use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_gossip::api::{GossipReceiver, GossipSender};
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;
use tokio::runtime::Runtime;

use super::NwChat;
use super::chat_connector::NwChatConnector;
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::{NwContact, NwProfile, chat_connector};
use tokio::sync::RwLock;



use tokio::runtime::Handle;

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
        // let test_error: anyhow::Result<()> = Err(anyhow::anyhow!("this is the root error"))
        //     .with_context(|| "this is the context");

        // test_error?;
        let db_client = Arc::unwrap_or_clone(db_client);
        let profile = db_client
            .get_nw_profile()
            .map_err(anyhow::Error::from)
            .with_context(|| "invalid private key")?;
        log::info!("2");
        let runtime = tokio::runtime::Runtime::new()
            .map_err(anyhow::Error::from)?;
        let runtime_handle = runtime.handle().clone();
        log::info!("3");
        let (gossip, router, chat_connectors) = runtime_handle.block_on(async {
            let endpoint = Endpoint::builder(presets::N0)
                .secret_key(profile.secret_key.clone())
                .alpns(vec![iroh_gossip::ALPN.to_vec()])
                .bind()
                .await
                .map_err(anyhow::Error::from)
                .with_context(|| "endpoint failed to bind")?;
            log::info!("4");
            let gossip = Gossip::builder().spawn(endpoint.clone());
            log::info!("5");
            let router = Router::builder(endpoint)
                .accept(iroh_gossip::ALPN, gossip.clone())
                .spawn();
            log::info!("6");
            let chats = db_client.get_nw_chats().await?;
            let mut chat_connectors = HashMap::with_capacity(chats.len());
            log::info!("7");
            for chat in chats {
                let topic_id = chat.topic_id;
                log::info!("before spawning chat: {:?}", &chat);
                let connector =
                    NwChatConnector::spawn(chat, db_client.clone(), &gossip, profile.secret_key.clone()).await
                        .with_context(|| "chat connector failed to spawn")?;
                log::info!("after spawning chat");
                chat_connectors.insert(topic_id, connector);
            }

            Ok::<_, FfiError>((gossip, router, chat_connectors))
        })?;

        //keeps runtime open so background tasks continue
        //To-Do: find out if this is necessary
        std::thread::spawn(move || {
            runtime.block_on(std::future::pending::<()>());
        });

        Ok(Arc::new(NwCore {
            runtime_handle,
            db_client,
            gossip,
            router,
            profile,
            chat_connectors,
        }))
    }

    fn send(&self, content: String, chat_id: String) -> Result<(), FfiError> {
        let mut topic_id_bytes = [0u8; 32];
        hex::decode_to_slice(&chat_id, &mut topic_id_bytes).map_err(anyhow::Error::from)?;
        let topic_id = TopicId::from_bytes(topic_id_bytes);

        
        let connector = self.chat_connectors
            .get(&topic_id)
            .ok_or_else(|| anyhow::anyhow!("No active connector for topic ID: {}", chat_id))?;

        self.runtime_handle.block_on(async {
            connector.send(content).await
        }).map_err(anyhow::Error::from)?;

        Ok(())
    }
}