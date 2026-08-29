use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};

use futures_lite::StreamExt;

use iroh::{Endpoint, EndpointAddr, EndpointId, endpoint, protocol::Router};

use iroh_gossip::{
    api::{Event, GossipReceiver}, net::Gossip, proto::{TopicId, topic},
};
use iroh::{endpoint::presets, SecretKey};

use std::println;

use serde::{Deserialize, Serialize};

use crate::nw::{NwChat, NwProfile, profile};

use super::{NwDbManager, NwDbClient};

use tokio::{runtime};


use super::{NwError, SetupErr};

use crate::run::parse_path;


struct NwContext
{
    chats: NwChat
}

#[derive(uniffi::Object)]
struct NwNode
{
    runtime: runtime::Runtime,
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    db_manager: NwDbManager
}

// #[uniffi::export]
async fn set_profile(profile_client: &NwDbClient) -> anyhow::Result<()> {
    let key = SecretKey::generate();
    let profile = NwProfile{
        secret_key: key.to_bytes().to_vec()
    };
    profile_client.set_profile(profile).await?;
    Ok(())
}

// #[uniffi::export]
fn start_nw(db_path: String) -> anyhow::Result<NwNode, NwError> {

    let db_manager = NwDbManager::spawn(parse_path( &db_path))
        .map_err(|e| NwError::InternalError {
            msg: format!("{}", e),
        })?;

    let profile_client = db_manager.create_client();

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| NwError::InternalError {
            msg: format!("failed to create runtime: {}", e),
        })?;

    let (endpoint, gossip, router) = runtime.block_on(async {
        let profile: NwProfile = match profile_client.get_profile().await {
            Ok(profile) => profile,

            Err(e) => {
                if let Some(SetupErr::ProfileNotSet) = e.downcast_ref::<SetupErr>() {
                    set_profile(&profile_client)
                        .await
                        .map_err(|e| NwError::InternalError {
                            msg: format!("{}", e),
                        })?;

                    profile_client
                        .get_profile()
                        .await
                        .map_err(|e| NwError::InternalError {
                            msg: format!("{}", e),
                        })?
                } else {
                    return Err(NwError::InternalError {
                        msg: format!("{}", e),
                    });
                }
            }
        };

        let secret_key: [u8; 32] = profile
            .secret_key
            .as_slice()
            .try_into()
            .map_err(|e| NwError::InternalError {
                msg: format!("{}", e),
            })?;

        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(SecretKey::from_bytes(&secret_key))
            .alpns(vec![iroh_gossip::ALPN.to_vec()])
            .bind()
            .await
            .map_err(|e| NwError::InternalError {
                msg: format!("{}", e),
            })?;

        let gossip = Gossip::builder().spawn(endpoint.clone());

        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        Ok::<(Endpoint, Gossip, Router), NwError>((endpoint, gossip, router))
    })?;

    let chats_client = db_manager.create_client();

    let chats = chats_client.get_chats().await?;

    Ok(NwNode {
        runtime,
        endpoint,
        gossip,
        router,
        db_manager,
    })
}

async fn 

