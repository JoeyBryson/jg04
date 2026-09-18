use iroh::{EndpointId, SecretKey, Signature};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use iroh_gossip::api::Message as GossipMessage;

///For receiving messages, we need to know who the message is from
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDataAndSender {
    pub content: String,
    pub sent_at: u64,
    pub sender: EndpointId
}

///For sending messages ourselves, the sender (us) is already known 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub content: String,
    pub sent_at: u64,
}

///message form passed to/ received from the iroh-gossip subscriber as bytes
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage {
    data: Vec<u8>,
    signature: Signature,
}

pub fn verify_and_decode(gossip_message: GossipMessage) -> Result<MessageDataAndSender> {
    let bytes = gossip_message.content;
    let signed_message: SignedMessage = postcard::from_bytes(&bytes)?;
    let sender = gossip_message.delivered_from;

    sender.verify(&signed_message.data, &signed_message.signature)?;

    let message: MessageData = postcard::from_bytes(&signed_message.data)?;

    Ok(MessageDataAndSender {
        content: message.content,
        sent_at: message.sent_at,
        sender,
    })
}

pub fn sign_and_encode(
    secret_key: &SecretKey,
    message_data: MessageData,
) -> Result<Vec<u8>> {
    let data = postcard::to_stdvec(&message_data)?;
    let signature = secret_key.sign(&data);

    let signed_message = SignedMessage {
        data,
        signature,
    };

    let encoded = postcard::to_stdvec(&signed_message)?;
    Ok(encoded)
}
