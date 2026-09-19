use anyhow::Result;
use iroh::{EndpointId, SecretKey};
use iroh_gossip::TopicId;

use super::client::DbClient;
use crate::network::{NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage};

fn sample_endpoint_id(seed: u8) -> EndpointId {
    SecretKey::from_bytes(&[seed; 32]).public()
}

impl DbClient {
    pub async fn add_sample_chat(&self) -> Result<()> {
        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: sample_endpoint_id(0),
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: sample_endpoint_id(1),
        };

        self.add_nw_contact(alice.clone()).await?;
        self.add_nw_contact(bob.clone()).await?;

        let topic_id = TopicId::from_bytes([2u8; 32]);

        let chat = NwChat {
            name: Some("Sample Chat".to_string()),
            members: vec![alice, bob]
                .into_iter()
                .map(|contact| NwChatMember {
                    contact,
                    status: NwChatMemberStatus::Joined,
                })
                .collect(),
            topic_id,
        };

        self.add_nw_chat(chat).await?;

        Ok(())
    }

    pub async fn add_sample_message(&self, time: i32) -> Result<()> {
        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: sample_endpoint_id(0),
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: sample_endpoint_id(1),
        };

        let topic_id = TopicId::from_bytes([100u8; 32]);

        let sent_at = 1779490800000i64 + 60000 * ((time as i64) + 300);

        let from_me = time % 3 == 0;

        let endpoint_id = if from_me {
            None
        } else if time % 2 == 0 {
            Some(alice.endpoint_id)
        } else {
            Some(bob.endpoint_id)
        };

        let message = NwMessage {
            topic_id,
            from_me,
            endpoint_id,
            content: format!("Sample message {}", 300 + time + 1),
            sent_at,
        };

        self.add_nw_message(message).await?;

        Ok(())
    }

    pub async fn add_sample_data(&self) -> Result<()> {
        let contacts: Vec<NwContact> = vec![
            "Angela", "Bob", "Charlie", "Diana", "Ethan", "Fiona", "George", "Hannah", "Isaac",
            "Julia",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let endpoint_id = sample_endpoint_id(i as u8);

            NwContact {
                name: name.to_string(),
                endpoint_id,
            }
        })
        .collect();

        // Add contacts
        for contact in &contacts {
            self.add_nw_contact(contact.clone()).await?;
        }

        for chat_index in 0..7 {
            let topic_id = TopicId::from_bytes([100 + chat_index as u8; 32]);

            let is_group = chat_index % 2 == 0;
            let count = if is_group { 4 } else { 2 };

            let contacts: Vec<_> = (0..count)
                .map(|i| contacts[(chat_index + i) % contacts.len()].clone())
                .collect();

            let members: Vec<_> = contacts
                .iter()
                .cloned()
                .map(|contact| NwChatMember {
                    contact,
                    status: NwChatMemberStatus::Joined,
                })
                .collect();

            let chat = NwChat {
                name: if is_group {
                    Some(format!("Group Chat {}", chat_index + 1))
                } else {
                    None
                },
                members: members.clone(),
                topic_id,
            };

            self.add_nw_chat(chat).await?;

            for message_index in 0..300 {
                let sent_at = 1779490800000i64
                    + (chat_index as i64 * 1_000_000)
                    + (message_index as i64 * 60_000);

                let from_me = message_index % 3 == 0;

                let endpoint_id = if from_me {
                    None
                } else {
                    let sender = &members[message_index as usize % members.len()];
                    Some(sender.contact.endpoint_id)
                };

                let message = NwMessage {
                    topic_id,
                    from_me,
                    endpoint_id,
                    content: format!("Sample message {}", message_index + 1),
                    sent_at,
                };

                self.add_nw_message(message).await?;
            }
        }

        Ok(())
    }
}
