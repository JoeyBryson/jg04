use std::{collections::HashMap, vec};
use iroh::{Endpoint, protocol::Router};
use iroh_gossip::{
    net::Gossip, proto::TopicId,
};
use iroh::endpoint::presets;
use tokio::{runtime};
use crate::ffi_error::FfiError;
use std::sync::Arc;
use super::chat_manager::ChatManager;
use crate::database::DbClient;

#[derive(uniffi::Object)]
struct NwCore {
    runtime: runtime::Runtime,
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    db_client: DbClient,
    chat_managers: HashMap<TopicId, ChatManager>,
}

#[uniffi::export]
impl NwCore {
    #[uniffi::constructor]
    fn spawn(db_client: Arc<DbClient>) -> Result<NwCore, FfiError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(anyhow::Error::from)?;

        let db_client_clone = (*db_client).clone();

        let (endpoint, gossip, router, chat_managers) = runtime.block_on(async {

            let profile = db_client_clone
                .get_nw_profile()
                .await?;

            let secret_key= profile.secret_key;

            let endpoint = Endpoint::builder(presets::N0)
                .secret_key(secret_key)
                .alpns(vec![iroh_gossip::ALPN.to_vec()])
                .bind()
                .await?;

            let gossip = Gossip::builder()
                .spawn(endpoint.clone());

            let router = Router::builder(endpoint.clone())
                .accept(iroh_gossip::ALPN, gossip.clone())
                .spawn();
            
            let chat_managers = Self::spawn_chat_managers(db_client_clone.clone(), &gossip)
                .await?;
            
            Ok::<_, anyhow::Error>((endpoint, gossip, router, chat_managers))
        })?;

        Ok(NwCore {
            runtime,
            endpoint,
            gossip,
            router,
            db_client: db_client_clone,
            chat_managers,
        })
    }
}

impl NwCore {
    async fn spawn_chat_managers(db_client: DbClient, gossip: &Gossip) -> anyhow::Result<HashMap<TopicId, ChatManager>>{
        let chats = db_client
                .get_nw_chats()
                .await?;

        let mut chat_managers = HashMap::new();

        for chat in chats {
            let chat_manager = ChatManager::spawn(chat.clone(), gossip, db_client.clone())
            .await?;

            chat_managers.insert(chat.topic_id, chat_manager);

        }

        Ok(chat_managers)
    }
}