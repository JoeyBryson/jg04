use anyhow::Context;
use iroh::Endpoint;
use iroh::protocol::Router;
use iroh_gossip::net::Gossip;
use tokio::runtime::Handle;

use super::{ChatInviteManager, NwCore};
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

        Self::finish_spawn(endpoint, db_client, profile).await
    }

    /// Like [`Self::spawn_base`], but binds the endpoint against a caller-supplied
    /// preset (a local relay/DNS pair) instead of the production `N0` defaults.
    /// Used by [`crate::harness`] to run entirely offline.
    pub(crate) async fn spawn_base_with_preset(
        db_client: DbClient,
        profile: NwProfile,
        preset: impl presets::Preset,
    ) -> Result<Self, FfiError> {
        let endpoint = Endpoint::builder(presets::Minimal)
            .preset(preset)
            .secret_key(profile.secret_key.clone())
            .alpns(vec![CONTROL_ALPN.to_vec(), iroh_gossip::ALPN.to_vec()])
            .bind()
            .await
            .map_err(anyhow::Error::from)
            .with_context(|| "endpoint failed to bind")?;

        Self::finish_spawn(endpoint, db_client, profile).await
    }

    async fn finish_spawn(
        endpoint: Endpoint,
        db_client: DbClient,
        profile: NwProfile,
    ) -> Result<Self, FfiError> {
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

        let chat_invite_actor = ChatInviteManager::spawn(router.clone(), db_client.clone());

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

