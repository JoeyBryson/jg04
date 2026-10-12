use anyhow::Result;
use iroh::{EndpointId, SecretKey, Signature};
use iroh_gossip::api::Message as GossipMessage;
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};

use crate::network::MAX_CHAT_MESSAGE_CONTENT_BYTES;

///For receiving messages, we need to know who the message is from
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDataAndSender {
    pub topic_id: TopicId,
    pub content: String,
    pub sent_at: u64,
    pub sender: EndpointId,
    pub message_id: [u8; 32],
    pub signed_payload: Vec<u8>,
    pub signature: Vec<u8>,
}

///For sending messages ourselves, the sender (us) is already known
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub content: String,
    pub sent_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Chat { content: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct SignedPayload {
    topic_id: TopicId,
    sent_at: u64,
    kind: MessageKind,
}

///message form passed to/ received from the iroh-gossip subscriber as bytes
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage {
    author: EndpointId,
    message_id: [u8; 32],
    data: Vec<u8>,
    signature: Signature,
}

pub fn verify_and_decode(
    gossip_message: GossipMessage,
    expected_topic_id: TopicId,
) -> Result<MessageDataAndSender> {
    verify_and_decode_bytes(&gossip_message.content, expected_topic_id)
}

// The author comes from the message itself; gossip's `delivered_from` is only the relaying neighbor.
fn verify_and_decode_bytes(
    bytes: &[u8],
    expected_topic_id: TopicId,
) -> Result<MessageDataAndSender> {
    let signed_message: SignedMessage = postcard::from_bytes(bytes)?;
    let sender = signed_message.author;

    sender.verify(&signed_message.data, &signed_message.signature)?;

    let computed_message_id = message_id(sender, &signed_message.data)?;
    if computed_message_id != signed_message.message_id {
        anyhow::bail!("signed message ID does not match its author and payload")
    }

    let payload: SignedPayload = postcard::from_bytes(&signed_message.data)?;
    if payload.topic_id != expected_topic_id {
        anyhow::bail!("signed message belongs to a different chat topic")
    }

    let content = match payload.kind {
        MessageKind::Chat { content } => content,
    };
    if content.len() > MAX_CHAT_MESSAGE_CONTENT_BYTES {
        anyhow::bail!("chat message content exceeds the maximum size")
    }

    Ok(MessageDataAndSender {
        topic_id: payload.topic_id,
        content,
        sent_at: payload.sent_at,
        sender,
        message_id: signed_message.message_id,
        signed_payload: signed_message.data,
        signature: signed_message.signature.to_bytes().to_vec(),
    })
}

pub fn sign_and_encode(
    secret_key: &SecretKey,
    topic_id: TopicId,
    message_data: MessageData,
) -> Result<(Vec<u8>, MessageDataAndSender)> {
    if message_data.content.len() > MAX_CHAT_MESSAGE_CONTENT_BYTES {
        anyhow::bail!("chat message content exceeds the maximum size")
    }

    let data = postcard::to_stdvec(&SignedPayload {
        topic_id,
        sent_at: message_data.sent_at,
        kind: MessageKind::Chat {
            content: message_data.content.clone(),
        },
    })?;
    let signature = secret_key.sign(&data);
    let author = secret_key.public();
    let message_id = message_id(author, &data)?;

    let signed_message = SignedMessage {
        author,
        message_id,
        data,
        signature,
    };

    let encoded = postcard::to_stdvec(&signed_message)?;
    let persisted_message = verify_and_decode_bytes(&encoded, topic_id)?;
    Ok((encoded, persisted_message))
}

fn message_id(author: EndpointId, signed_payload: &[u8]) -> Result<[u8; 32]> {
    let identity_bytes = postcard::to_stdvec(&(author, signed_payload))?;
    Ok(*blake3::hash(&identity_bytes).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message() -> MessageData {
        MessageData {
            content: "hello".to_string(),
            sent_at: 42,
        }
    }

    fn topic(seed: u8) -> TopicId {
        TopicId::from_bytes([seed; 32])
    }

    #[test]
    fn decodes_author_from_message_not_transport() {
        let key = SecretKey::from_bytes(&[1; 32]);
        let topic_id = topic(3);
        let (bytes, _) = sign_and_encode(&key, topic_id, message()).unwrap();

        let decoded = verify_and_decode_bytes(&bytes, topic_id).unwrap();

        assert_eq!(decoded.sender, key.public());
        assert_eq!(decoded.topic_id, topic_id);
        assert_eq!(decoded.content, "hello");
        assert_eq!(decoded.sent_at, 42);
    }

    #[test]
    fn rejects_signed_message_replayed_in_another_topic() {
        let key = SecretKey::from_bytes(&[1; 32]);
        let signed_topic = topic(3);
        let other_topic = topic(4);
        let (bytes, _) = sign_and_encode(&key, signed_topic, message()).unwrap();

        assert!(verify_and_decode_bytes(&bytes, other_topic).is_err());
    }

    #[test]
    fn rejects_message_with_swapped_author() {
        let key = SecretKey::from_bytes(&[1; 32]);
        let other = SecretKey::from_bytes(&[2; 32]);
        let (bytes, _) = sign_and_encode(&key, topic(3), message()).unwrap();

        let mut signed: SignedMessage = postcard::from_bytes(&bytes).unwrap();
        signed.author = other.public();
        let forged = postcard::to_stdvec(&signed).unwrap();

        assert!(verify_and_decode_bytes(&forged, topic(3)).is_err());
    }

    #[test]
    fn same_author_and_payload_produce_same_message_id() {
        let key = SecretKey::from_bytes(&[1; 32]);
        let topic_id = topic(3);

        let (_, first) = sign_and_encode(&key, topic_id, message()).unwrap();
        let (_, second) = sign_and_encode(&key, topic_id, message()).unwrap();

        assert_eq!(first.message_id, second.message_id);
        assert_eq!(first.signed_payload, second.signed_payload);
        assert_eq!(first.signature, second.signature);
    }

    #[test]
    fn message_id_changes_when_signed_payload_changes() {
        let key = SecretKey::from_bytes(&[1; 32]);
        let topic_id = topic(3);
        let (_, first) = sign_and_encode(&key, topic_id, message()).unwrap();
        let (_, second) = sign_and_encode(
            &key,
            topic_id,
            MessageData {
                content: "different".to_string(),
                sent_at: 42,
            },
        )
        .unwrap();

        assert_ne!(first.message_id, second.message_id);
    }

    #[test]
    fn enforces_chat_content_byte_limit_before_signing() {
        let key = SecretKey::from_bytes(&[1; 32]);
        let topic_id = topic(3);
        let at_limit = MessageData {
            content: "a".repeat(MAX_CHAT_MESSAGE_CONTENT_BYTES),
            sent_at: 42,
        };

        let (_, decoded) = sign_and_encode(&key, topic_id, at_limit).unwrap();
        assert_eq!(decoded.content.len(), MAX_CHAT_MESSAGE_CONTENT_BYTES);

        let over_limit = MessageData {
            content: "a".repeat(MAX_CHAT_MESSAGE_CONTENT_BYTES + 1),
            sent_at: 42,
        };
        assert!(sign_and_encode(&key, topic_id, over_limit).is_err());

        let multibyte_over_limit = MessageData {
            content: "é".repeat(MAX_CHAT_MESSAGE_CONTENT_BYTES / 2 + 1),
            sent_at: 42,
        };
        assert!(sign_and_encode(&key, topic_id, multibyte_over_limit).is_err());
    }
}
