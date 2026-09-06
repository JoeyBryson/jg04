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
        let db_client = Arc::unwrap_or_clone(db_client);
        let profile = db_client
            .get_nw_profile()
            .map_err(anyhow::Error::from)?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(anyhow::Error::from)?;
        let runtime_handle = runtime.handle().clone();

        let (gossip, router, chat_connectors) = runtime_handle.block_on(async {
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

            let chats = db_client.get_nw_chats().await?;
            let mut chat_connectors = HashMap::with_capacity(chats.len());

            for chat in chats {
                let topic_id = chat.topic_id;
                let connector =
                    NwChatConnector::spawn(chat, db_client.clone(), &gossip).await?;

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
}