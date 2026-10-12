use std::time::Duration;

use anyhow::{Result, bail};
use futures_lite::StreamExt;
use iroh::SecretKey;
use iroh_gossip::api::{Event as GossipEvent, GossipTopic};
use iroh_gossip::proto::TopicId;

mod common;
use common::TestNetwork;

async fn wait_for_neighbor_event(
    topic: &mut GossipTopic,
    expected_peer: iroh::EndpointId,
    up: bool,
) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let Some(event) = topic.try_next().await? else {
                bail!("gossip topic stream ended before neighbor event")
            };

            match event {
                GossipEvent::NeighborUp(peer) if up && peer == expected_peer => {
                    tracing::info!(peer = %peer, "observed gossip NeighborUp");
                    return Ok(());
                }
                GossipEvent::NeighborDown(peer) if !up && peer == expected_peer => {
                    tracing::info!(peer = %peer, "observed gossip NeighborDown");
                    return Ok(());
                }
                other => tracing::debug!(event = ?other, "ignoring unrelated gossip event"),
            }
        }
    })
    .await??;

    Ok(())
}

#[test]
fn neighbor_up_fires_on_both_sides_and_after_reconnect() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "debug,iroh=off,iroh_gossip=info,iroh_relay=off,portmapper=off,igd_next=off,reqwest=off,hyper=off,hyper_util=off,rustls=off,hickory_net=off,hickory_resolver=off,noq_proto=off",
                )
            }),
        )
        .try_init();

    tracing::info!("starting G1 gossip neighbor integration test");
    let network = TestNetwork::spawn()?;
    let alice_key = SecretKey::from_bytes(&[71; 32]);
    let bob_key = SecretKey::from_bytes(&[72; 32]);
    let topic_id = TopicId::from_bytes([73; 32]);
    tracing::info!(topic = %&topic_id.to_string()[..10], "local test network ready");

    network.runtime().block_on(async {
        let alice = network.spawn_gossip_peer(alice_key).await?;
        let bob = network.spawn_gossip_peer(bob_key.clone()).await?;
        tracing::info!(alice = %alice.endpoint_id, bob = %bob.endpoint_id, "gossip peers started");

        let mut alice_topic = alice.gossip.subscribe(topic_id, vec![]).await?;
        let mut bob_topic = bob
            .gossip
            .subscribe(topic_id, vec![alice.endpoint_id])
            .await?;
        tracing::info!("both peers subscribed; waiting for initial neighbor events");

        wait_for_neighbor_event(&mut alice_topic, bob.endpoint_id, true).await?;
        wait_for_neighbor_event(&mut bob_topic, alice.endpoint_id, true).await?;
        tracing::info!("both peers observed the initial NeighborUp");

        let bob_endpoint_id = bob.endpoint_id;
        drop(bob_topic);
        bob.shutdown().await?;
        tracing::info!(peer = %bob_endpoint_id, "Bob went offline");
        wait_for_neighbor_event(&mut alice_topic, bob_endpoint_id, false).await?;

        let bob = network.spawn_gossip_peer(bob_key).await?;
        let mut bob_topic = bob
            .gossip
            .subscribe(topic_id, vec![alice.endpoint_id])
            .await?;
        tracing::info!(peer = %bob.endpoint_id, "Bob restarted with the same identity");

        wait_for_neighbor_event(&mut alice_topic, bob.endpoint_id, true).await?;
        wait_for_neighbor_event(&mut bob_topic, alice.endpoint_id, true).await?;
        tracing::info!("both peers observed NeighborUp after reconnect");

        drop(bob_topic);
        drop(alice_topic);
        bob.shutdown().await?;
        alice.shutdown().await?;
        tracing::info!("G1 gossip neighbor integration test completed");
        Ok(())
    })
}
