use std::collections::HashMap;
use std::sync::Arc;

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
use crate::network::{NwContact, NwProfile};
use tokio::sync::RwLock;



use tokio::runtime::Handle;

#[derive(uniffi::Object)]
pub struct NwCore {
    runtime_handle: Handle,
    db_client: Arc<DbClient>,
    gossip: Gossip,
    router: Router,
    profile: NwProfile,
    activity: RwLock<HashMap<TopicId, NwChatConnector>>,
}

#[uniffi::export]
impl NwCore {
    #[uniffi::constructor]
    pub fn spawn(db_client: Arc<DbClient>) -> Result<Arc<Self>, FfiError> {
        let db_client_inner = Arc::unwrap_or_clone(db_client);
        let profile = db_client_inner.get_nw_profile().map_err(anyhow::Error::from)?;

        let runtime = tokio::runtime::Runtime::new().map_err(anyhow::Error::from)?;
        let runtime_handle = runtime.handle().clone();

        // Boot network components using the runtime handle
        let (gossip, router) = runtime_handle.block_on(async {
            let endpoint = Endpoint::builder(presets::N0)
                .secret_key(profile.secret_key.clone())
                .alpns(vec![iroh_gossip::ALPN.to_vec()])
                .bind()
                .await
                .map_err(anyhow::Error::from)?;

            let gossip = Gossip::builder().spawn(endpoint.clone());

            let router = Router::builder(endpoint.clone())
                .accept(iroh_gossip::ALPN, gossip.clone())
                .spawn();

            Ok::<(Gossip, Router), FfiError>((gossip, router))
        })?;

        // Park the Tokio runtime in a dedicated OS thread so background tasks stay active
        //To-Do: investigate if this is really needed
        std::thread::spawn(move || {
            runtime.block_on(std::future::pending::<()>());
        });

        Ok(Arc::new(NwCore {
            runtime_handle,
            db_client: db_client.clone(),
            gossip,
            router,
            profile,
            activity: RwLock::new(HashMap::new()),
        }))
    }
}

impl NwCore {
    async fn spawn_chat_connectors(
        db_client: &DbClient,
        gossip: &Gossip,
    ) -> anyhow::Result<HashMap<TopicId, NwChatConnector>> {
        let chats = db_client.get_nw_chats().await?;
        let mut chat_connectors = HashMap::with_capacity(chats.len());

        for chat in chats {
            let topic_id = chat.topic_id;
            
            // Instantiates connector instantly, then connects asynchronously.
            let mut connector = NwChatConnector::new(chat, db_client.clone());
            let _ = connector.connect(gossip).await; // Handle error gracefully/retry in background

            chat_connectors.insert(topic_id, connector);
        }

        Ok(chat_connectors)
    }

    /// 2. Periodically checks connection status across all managed topics.
    pub async fn health_check_connectors(&mut self) {
        todo!("Iterate over chat_managers and invoke sync_peer_state or attempt reconnection")
    }
}