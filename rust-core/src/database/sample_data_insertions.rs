//! This file is for early stage testing and should be deleted soon once our
//! testing setup has been improved.

use anyhow::Result;
use iroh::{EndpointId, SecretKey};
use iroh_gossip::TopicId;

use super::client::DbClient;
use crate::network::{
    NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile,
};

fn sample_endpoint_id(seed: u8) -> EndpointId {
    SecretKey::from_bytes(&[seed; 32]).public()
}

fn sample_profile() -> NwProfile {
    let secret_key = SecretKey::from_bytes(&[42; 32]);

    NwProfile {
        contact: NwContact {
            name: "Me".to_string(),
            endpoint_id: secret_key.public(),
        },
        secret_key,
    }
}

impl DbClient {
    pub async fn add_sample_chat(&self) -> Result<()> {
        let profile = sample_profile();

        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: sample_endpoint_id(0),
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: sample_endpoint_id(1),
        };

        self.set_profile(profile.clone())?;
        self.add_nw_contact(alice.clone()).await?;
        self.add_nw_contact(bob.clone()).await?;

        let topic_id = TopicId::from_bytes([2u8; 32]);

        let members = vec![
            NwChatMember {
                contact: profile.contact,
                status: NwChatMemberStatus::Joined,
            },
            NwChatMember {
                contact: alice,
                status: NwChatMemberStatus::Joined,
            },
            NwChatMember {
                contact: bob,
                status: NwChatMemberStatus::Joined,
            },
        ];

        self.add_nw_chat(NwChat {
            name: Some("Sample Chat".to_string()),
            members,
            topic_id,
        })
        .await?;

        Ok(())
    }

    pub async fn add_sample_message(&self, time: i32) -> Result<()> {
        let profile = sample_profile();

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

        let endpoint_id = if time % 3 == 0 {
            profile.contact.endpoint_id
        } else if time % 2 == 0 {
            alice.endpoint_id
        } else {
            bob.endpoint_id
        };

        let message = NwMessage {
            topic_id,
            endpoint_id,
            content: format!("Sample message {}", 300 + time + 1),
            sent_at,
        };

        self.add_nw_message(message).await?;

        Ok(())
    }

    pub async fn add_sample_data(&self) -> Result<()> {
        let profile = sample_profile();

        self.set_profile(profile.clone())?;

        let contacts = vec![
            profile.contact.clone(),
            NwContact {
                name: "Angela".to_string(),
                endpoint_id: sample_endpoint_id(0),
            },
            NwContact {
                name: "Bob".to_string(),
                endpoint_id: sample_endpoint_id(1),
            },
            NwContact {
                name: "Charlie".to_string(),
                endpoint_id: sample_endpoint_id(2),
            },
            NwContact {
                name: "Diana".to_string(),
                endpoint_id: sample_endpoint_id(3),
            },
            NwContact {
                name: "Ethan".to_string(),
                endpoint_id: sample_endpoint_id(4),
            },
            NwContact {
                name: "Fiona".to_string(),
                endpoint_id: sample_endpoint_id(5),
            },
            NwContact {
                name: "George".to_string(),
                endpoint_id: sample_endpoint_id(6),
            },
            NwContact {
                name: "Hannah".to_string(),
                endpoint_id: sample_endpoint_id(7),
            },
            NwContact {
                name: "Isaac".to_string(),
                endpoint_id: sample_endpoint_id(8),
            },
            NwContact {
                name: "Julia".to_string(),
                endpoint_id: sample_endpoint_id(9),
            },
        ];

        for contact in &contacts {
            self.add_nw_contact(contact.clone()).await?;
        }

        for chat_index in 0..7 {
            let topic_id = TopicId::from_bytes([100 + chat_index as u8; 32]);

            let is_group = chat_index % 2 == 0;
            let other_member_count = if is_group { 4 } else { 2 };

            let mut members = vec![NwChatMember {
                contact: profile.contact.clone(),
                status: NwChatMemberStatus::Joined,
            }];

            members.extend(
                (0..other_member_count)
                    .map(|i| contacts[(chat_index + i + 1) % contacts.len()].clone())
                    .map(|contact| NwChatMember {
                        contact,
                        status: NwChatMemberStatus::Joined,
                    }),
            );

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

                let endpoint_id = if message_index % 3 == 0 {
                    profile.contact.endpoint_id
                } else {
                    members[message_index as usize % members.len()]
                        .contact
                        .endpoint_id
                };

                let message = NwMessage {
                    topic_id,
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