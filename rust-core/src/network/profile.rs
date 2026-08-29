use std::{collections::HashMap, fmt, str::FromStr};

use anyhow::Result;
use futures_lite::StreamExt;

use iroh::{protocol::Router, Endpoint, EndpointAddr, EndpointId};

use iroh_gossip::{
    api::{GossipReceiver, Event},
    net::Gossip,
    proto::TopicId,
};
use serde::{Deserialize, Serialize};

// async fn get_endpoint() -> Endpoint {
    
// }