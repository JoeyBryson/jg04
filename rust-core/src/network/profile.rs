use std::{collections::HashMap, fmt, str::FromStr};

use anyhow::Result;
use futures_lite::StreamExt;

use iroh::{
    protocol::Router, Endpoint, EndpointAddr, EndpointId, SecretKey,
};

use iroh_gossip::{
    api::{GossipReceiver, Event},
    net::Gossip,
    proto::TopicId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::ffi_error::FfiError;
use crate::network::NwProfile;
use crate::network::NwDbClient;


#[uniffi::export]
fn set_secret_key(db_client: Arc<NwDbClient>) -> Result<(), FfiError> {
    
    let secret_key = SecretKey::generate();
    tokio::runtime::Runtime::new()
        .map_err(anyhow::Error::from)?
        .block_on(db_client
        .set_profile(NwProfile {
                secret_key
            }
        )
    )?;
    
    Ok(())
}