use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};
use futures_lite::StreamExt;
use iroh::{Endpoint, EndpointAddr, EndpointId, endpoint, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender}, net::Gossip, proto::{TopicId, topic},
};
use iroh::{endpoint::presets, SecretKey};
use std::println;
use serde::{Deserialize, Serialize};
use super::{NwChat, NwProfile, SetupError};
use tokio::{runtime};
use crate::{ffi_error::FfiError, database::sample_data_insertions};
use std::sync::Arc;
use crate::database::DbClient;
use crate::network::NwContact;

pub struct ChatManager {
    pub db_client: DbClient,
    pub topic_id: TopicId,
    pub members: Vec<NwContact>,
    pub sender: GossipSender,
    pub receiver: GossipReceiver,
}

impl ChatManager {
    pub async fn spawn(chat: NwChat, gossip: &Gossip, db_client: DbClient) -> anyhow::Result<Self> {
            let topic_id = chat.topic_id;

            let members = chat.members;

            let bootstrap_ids = members
                .iter()
                .map(|member| member.endpoint_id)
                .collect::<Vec<EndpointId>>();
            

            let (sender, receiver) = gossip.subscribe_and_join(
                topic_id, 
                bootstrap_ids
            ).await?.split();
            

            Ok(Self {
                db_client,
                topic_id,
                members,
                sender,
                receiver
            })
    }
}