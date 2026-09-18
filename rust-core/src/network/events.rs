
use iroh::EndpointId;
use serde::{Deserialize, Serialize};

use super::signed_message::{SignedMessage};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NwEvent {
    #[serde(rename_all = "camelCase")]
    MessageReceived {
        signed_message: SignedMessage
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

// impl TryFrom<GossipEvent> for NwEvent {
//     type Error = anyhow::Error;
//     fn try_from(event: GossipEvent) -> Result<Self, Self::Error> {
//         let converted = match event {
//             GossipEvent::NeighborUp(endpoint_id) => Self::NeighborUp { endpoint_id },
//             GossipEvent::NeighborDown(endpoint_id) => Self::NeighborDown { endpoint_id },
//             GossipEvent::Received(message) => {
//                 let message = SignedMessage::verify_and_decode(&message.content)
//                     .context("failed to parse and verify signed message")?;
//                 match message.message {
//                     Message::Message { text, nickname } => Self::MessageReceived {
//                         from: message.sender,
//                         text,
//                         nickname,
//                         sent_timestamp: message.timestamp,
//                     },
//                 }
//             }
//             GossipEvent::Lagged => Self::Lagged,
//         };
//         Ok(converted)
//     }
// }