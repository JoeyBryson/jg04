use anyhow::Context;
use iroh::Endpoint;
use iroh::protocol::Router;
use iroh_gossip::net::Gossip;
use tokio::runtime::Handle;

use super::{ChatInviteActor, NwCore};
use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use iroh::endpoint::presets;

use super::super::{CONTROL_ALPN, ChatSessionManager, ControlProtocol, NwProfile};

impl NwCore {
    pub async fn spawn_base(db_client: DbClient, profile: NwProfile) -> Result<Self, FfiError> {
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(profile.secret_key.clone())
            .alpns(vec![CONTROL_ALPN.to_vec(), iroh_gossip::ALPN.to_vec()])
            .bind()
            .await
            .map_err(anyhow::Error::from)
            .with_context(|| "endpoint failed to bind")?;

        let gossip = Gossip::builder().spawn(endpoint.clone());

        let chat_session_manager = ChatSessionManager::spawn(
            db_client.clone(),
            gossip.clone(),
            profile.secret_key.clone(),
        );

        let control_protocol = ControlProtocol {
            profile: profile.clone(),
            db_client: db_client.clone(),
            chat_session_manager: chat_session_manager.clone(),
        };

        let router = Router::builder(endpoint)
            .accept(CONTROL_ALPN, control_protocol)
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let chat_invite_actor = ChatInviteActor::spawn(router.clone(), db_client.clone());

        Ok(NwCore {
            runtime_handle: Handle::current(),
            db_client,
            gossip,
            router,
            profile,
            chat_session_manager,
            chat_invite_actor,
        })
    }
}
