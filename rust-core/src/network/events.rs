
use anyhow::{Context, Result};
use iroh::EndpointId;
use iroh_gossip::api::Event as GossipEvent;
use serde::{Deserialize, Serialize};
use super::signed_message::{Message, WireMessage, ReceivedMessage, SignedMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NwEvent {
    #[serde(rename_all = "camelCase")]
    Joined {
        neighbors: Vec<EndpointId>,
    },
    #[serde(rename_all = "camelCase")]
    MessageReceived {
        from: EndpointId,
        text: String,
        nickname: String,
        sent_timestamp: u64,
    },
    #[serde(rename_all = "camelCase")]
    Presence {
        from: EndpointId,
        nickname: String,
        sent_timestamp: u64,
    },
    #[serde(rename_all = "camelCase")]
    NeighborUp {
        endpoint_id: EndpointId,
    },
    #[serde(rename_all = "camelCase")]
    NeighborDown {
        endpoint_id: EndpointId,
    },
    Lagged,
}

impl TryFrom<GossipEvent> for NwEvent {
    type Error = anyhow::Error;
    fn try_from(event: GossipEvent) -> Result<Self, Self::Error> {
        let converted = match event {
            GossipEvent::NeighborUp(endpoint_id) => Self::NeighborUp { endpoint_id },
            GossipEvent::NeighborDown(endpoint_id) => Self::NeighborDown { endpoint_id },
            GossipEvent::Received(message) => {
                let message = SignedMessage::verify_and_decode(&message.content)
                    .context("failed to parse and verify signed message")?;
                match message.message {
                    Message::Presence { nickname } => Self::Presence {
                        from: message.from,
                        nickname,
                        sent_timestamp: message.timestamp,
                    },
                    Message::Message { text, nickname } => Self::MessageReceived {
                        from: message.from,
                        text,
                        nickname,
                        sent_timestamp: message.timestamp,
                    },
                }
            }
            GossipEvent::Lagged => Self::Lagged,
        };
        Ok(converted)
    }
}