use iroh::EndpointId;
use anyhow::Result;
use iroh_gossip::TopicId;

use super::{NwChat, NwContact, DbClient, NwMessage};

impl DbClient {
    pub async fn add_sample_chat(&self) -> Result<()> {
        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: EndpointId::from_bytes(&[0u8; 32])?,
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: EndpointId::from_bytes(&[1u8; 32])?,
        };

        self.add_nw_contact(alice.clone()).await?;
        self.add_nw_contact(bob.clone()).await?;

        let topic_id = TopicId::from_bytes([2u8; 32]);

        let chat = NwChat {
            name: Some("Sample Chat".to_string()),
            members: vec![alice, bob],
            topic_id,
        };

        self.add_nw_chat(chat).await?;

        Ok(())
    }

    pub async fn add_sample_message(&self, time: i32) -> Result<()> {
        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: EndpointId::from_bytes(&[0u8; 32])?,
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: EndpointId::from_bytes(&[1u8; 32])?,
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
            "Angela",
            "Bob",
            "Charlie",
            "Diana",
            "Ethan",
            "Fiona",
            "George",
            "Hannah",
            "Isaac",
            "Julia",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let endpoint_id = EndpointId::from_bytes(&[i as u8; 32])?;

            Ok(NwContact {
                name: name.to_string(),
                endpoint_id,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

        // Add contacts
        for contact in &contacts {
            self.add_nw_contact(contact.clone()).await?;
        }

        for chat_index in 0..7 {
            let topic_id = TopicId::from_bytes([100 + chat_index as u8; 32]);

            let is_group = chat_index % 2 == 0;
            let count = if is_group { 4 } else { 2 };

            let members: Vec<_> = (0..count)
                .map(|i| contacts[(chat_index + i) % contacts.len()].clone())
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
                let sent_at =
                    1779490800000i64
                    + (chat_index as i64 * 1_000_000)
                    + (message_index as i64 * 60_000);

                let from_me = message_index % 3 == 0;

                let endpoint_id = if from_me {
                    None
                } else {
                    let sender = &members[message_index as usize % members.len()];
                    Some(sender.endpoint_id)
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
