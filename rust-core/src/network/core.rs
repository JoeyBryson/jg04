use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};
use futures_lite::StreamExt;
use iroh::{Endpoint, EndpointAddr, EndpointId, endpoint, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipReceiver}, net::Gossip, proto::{TopicId, topic},
};
use iroh::{endpoint::presets, SecretKey};
use std::println;
use serde::{Deserialize, Serialize};
use super::{NwDbManager, NwDbClient, NwChat, NwProfile, SetupError};
use tokio::{runtime};
use crate::run::parse_path;
use crate::ffi_error::FfiError;



#[derive(uniffi::Object)]
struct NwCore {
    runtime: runtime::Runtime,
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    db_manager: NwDbManager,
}
 
impl NwCore {
    async fn start(
        db_manager: &NwDbManager,
        secret_key: SecretKey,
    ) -> Result<(Endpoint, Gossip, Router), anyhow::Error> {
        print!("lol");
        print!("lol");
        print!("lol");
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(secret_key)
            .alpns(vec![iroh_gossip::ALPN.to_vec()])
            .bind()
            .await?;

        let gossip = Gossip::builder().spawn(endpoint.clone());

        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        db_manager
            .create_client()
            .get_chats()
            .await?;

        Ok((endpoint, gossip, router))
    }

    #[uniffi::constructor]
    fn spawn(db_path: String) -> Result<NwCore, FfiError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(anyhow::Error::from)?;

        let db_manager = NwDbManager::spawn(parse_path(&db_path))?;

        let (endpoint, gossip, router) = runtime.block_on(async {
            let profile = db_manager
                .create_client()
                .get_profile()
                .await?;

            let secret_key: [u8; 32] = profile
                .secret_key
                .as_slice()
                .try_into()?;

            Self::start(
                &db_manager,
                SecretKey::from_bytes(&secret_key),
            )
            .await
        })?;

        Ok(NwCore {
            runtime,
            endpoint,
            gossip,
            router,
            db_manager,
        })
    }

    #[uniffi::constructor]
    fn initialize_and_spawn(db_path: String) -> Result<NwCore, FfiError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(anyhow::Error::from)?;

        let db_manager = NwDbManager::spawn(parse_path(&db_path))?;

        let (endpoint, gossip, router) = runtime.block_on(async {
            let secret_key = SecretKey::generate();

            db_manager
                .create_client()
                .set_profile(NwProfile {
                    secret_key: secret_key.to_bytes().to_vec(),
                })
                .await?;

            Self::start(&db_manager, secret_key).await
        })?;

        Ok(NwCore {
            runtime,
            endpoint,
            gossip,
            router,
            db_manager,
        })
    }
}

// fn spawn(db_path: String) -> Result<NwCore, FfiError> {
//     let runtime = tokio::runtime::Runtime::new()
//         .map_err(anyhow::Error::from)?;

//     let (endpoint, gossip, router, db_manager) = runtime.block_on(async {
//         let db_manager = NwDbManager::spawn(parse_path(&db_path))?;

//         let profile_client = db_manager.create_client();

//         let profile = match profile_client.get_profile().await {
//             Ok(profile) => profile,
//             Err(e) if e.downcast_ref::<SetupError>() == Some(&SetupError::ProfileNotSet) => {
//                 set_profile(&profile_client).await?;
//                 profile_client.get_profile().await?
//             }
//             Err(e) => return Err(e),
//         };

//         let secret_key: [u8; 32] = profile
//             .secret_key
//             .as_slice()
//             .try_into()?;

//         let endpoint = Endpoint::builder(presets::N0)
//             .secret_key(SecretKey::from_bytes(&secret_key))
//             .alpns(vec![iroh_gossip::ALPN.to_vec()])
//             .bind()
//             .await?;

//         let gossip = Gossip::builder().spawn(endpoint.clone());

//         let router = Router::builder(endpoint.clone())
//             .accept(iroh_gossip::ALPN, gossip.clone())
//             .spawn();

//         let chats_client = db_manager.create_client();
//         chats_client.get_chats().await?;

//         Ok((endpoint, gossip, router, db_manager))
//     })?;

//     Ok(NwCore {
//         runtime,
//         endpoint,
//         gossip,
//         router,
//         db_manager,
//     })
// }

// async fn set_profile(profile_client: &NwDbClient) -> anyhow::Result<()> {
//     let key = SecretKey::generate();

//     let profile = NwProfile {
//         secret_key: key.to_bytes().to_vec(),
//     };

//     profile_client.set_profile(profile).await?;

//     Ok(())
// }