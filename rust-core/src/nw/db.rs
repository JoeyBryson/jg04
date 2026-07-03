use tokio::sync::oneshot;
use anyhow::{Context, Result};
use hex;

use super::{NwChat, NwContact, NwDbClient, NwDbRequest, NwMessage};
use crate::notifications::{UiEvent, emit_ui_event};
use super::{NwDbManager};

impl NwDbClient {
    async fn send_request<T>(
        &self,
        request: NwDbRequest,
        rx: oneshot::Receiver<Result<T>>,
    ) -> Result<T> {
        self.worker_tx
            .send(request)
            .await
            .with_context(|| "NW002 send error")?;

        let res = rx
            .await
            .with_context(|| "NW003 recv error")?
            .with_context(|| "NW004 DB error")?;

        Ok(res)
    }

    pub async fn add_message(&self, message: NwMessage) -> Result<()> {
        let topic_id = message.topic_id.clone();
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::AddMessage {
                message,
                reply: tx,
            },
            rx,
        )
        .await?;

        emit_ui_event(UiEvent::ChatDataChanged { 
            topic_id: hex::encode(topic_id) 
        });
        emit_ui_event(UiEvent::ChatHeadersChanged);

        Ok(())
    }

    pub async fn add_contact(&self, contact: NwContact) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::AddContact {
                contact,
                reply: tx,
            },
            rx,
        )
        .await?;

        emit_ui_event(UiEvent::ContactsChanged);

        Ok(())
    }

    pub async fn add_chat(&self, chat: NwChat) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::AddChat {
                chat,
                reply: tx,
            },
            rx,
        )
        .await?;

        emit_ui_event(UiEvent::ChatHeadersChanged);

        Ok(())
    }

    pub async fn get_chats(&self) -> Result<Vec<NwChat>> {
        let (tx, rx) = oneshot::channel();
        self.send_request(NwDbRequest::GetChats { reply: tx }, rx).await
    }

    pub async fn get_chat_members(&self, topic_id: Vec<u8>) -> Result<Vec<NwContact>> {
        let (tx, rx) = oneshot::channel();
        self.send_request(
            NwDbRequest::GetChatMembers { topic_id, reply: tx },
            rx,
        )
        .await
    }

    pub async fn get_chat(&self, topic_id: Vec<u8>) -> Result<NwChat> {
        let (tx, rx) = oneshot::channel();
        self.send_request(NwDbRequest::GetChat { topic_id, reply: tx }, rx).await
    }

    pub async fn get_chat_messages(&self, topic_id: Vec<u8>) -> Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();
        self.send_request(
            NwDbRequest::GetChatMessages { topic_id, reply: tx },
            rx,
        )
        .await
    }

    pub async fn get_messages(&self) -> Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();
        self.send_request(NwDbRequest::GetMessages { reply: tx }, rx).await
    }

    pub async fn add_sample_chat(&self) -> Result<()> {
        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: vec![0u8; 32],
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: vec![1u8; 32],
        };

        self.add_contact(alice.clone()).await?;
        self.add_contact(bob.clone()).await?;

        let topic_id = vec![2u8; 32];

        let chat = NwChat {
            name: Some("Sample Chat".to_string()),
            members: vec![alice.clone(), bob.clone()],
            topic_id: topic_id.clone(),
        };

        self.add_chat(chat).await?;

        Ok(())
    }

    pub async fn add_sample_message(&self, time: i32) -> Result<()> {
        let alice = NwContact {
            name: "Angela".to_string(),
            endpoint_id: vec![0u8; 32],
        };

        let bob = NwContact {
            name: "Bob".to_string(),
            endpoint_id: vec![1u8; 32],
        };

        let topic_id = vec![100 as u8; 32];
        let sent_at = 1779490800000i64 + 60000 * (((time as i64) +300));
        // let sent_at =
        //             1779490800000i64
        //             + (chat_index as i64 * 1_000_000)
        //             + (message_index as i64 * 60_000)
        let from_me = time % 3 == 0;

        let endpoint_id = if from_me {
            None
        } else {
            Some(if time % 2 == 0 {
                alice.endpoint_id.clone()
            } else {
                bob.endpoint_id.clone()
            })
        };

        let message = NwMessage {
            topic_id: topic_id.clone(),
            from_me,
            endpoint_id,
            content: format!("Sample message {}", 300+time + 1),
            sent_at,
        };

        self.add_message(message).await?;

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
        .map(|(i, name)| NwContact {
            name: name.to_string(),
            endpoint_id: vec![i as u8; 32],
        })
        .collect();

        // add contacts
        for contact in &contacts {
            self.add_contact(contact.clone()).await?;
        }

        for chat_index in 0..7 {

            let topic_id = vec![100 + chat_index as u8; 32];

            let is_group = chat_index % 2 == 0;

            let members = if is_group {
                vec![
                    contacts[chat_index % contacts.len()].clone(),
                    contacts[(chat_index + 1) % contacts.len()].clone(),
                    contacts[(chat_index + 2) % contacts.len()].clone(),
                    contacts[(chat_index + 3) % contacts.len()].clone(),
                ]
            } else {
                vec![
                    contacts[chat_index % contacts.len()].clone(),
                    contacts[(chat_index + 1) % contacts.len()].clone(),
                ]
            };

            let chat = NwChat {
                name: if is_group {
                    Some(format!("Group Chat {}", chat_index + 1))
                } else {
                    None
                },
                members: members.clone(),
                topic_id: topic_id.clone(),
            };

            self.add_chat(chat).await?;

            for message_index in 0..300 {

                let sent_at =
                    1779490800000i64
                    + (chat_index as i64 * 1_000_000)
                    + (message_index as i64 * 60_000);

                let from_me = message_index % 3 == 0;

                let endpoint_id = if from_me {
                    None
                } else {

                    let sender =
                        &members[message_index as usize % members.len()];

                    Some(sender.endpoint_id.clone())
                };

                let message = NwMessage {
                    topic_id: topic_id.clone(),
                    from_me,
                    endpoint_id,
                    content: format!(
                        "Sample message {}",
                        message_index + 1
                    ),
                    sent_at,
                };

                self.add_message(message).await?;
            }
        }

        Ok(())
    }

}



impl NwDbManager {
    pub fn create_client(
        &self
    ) -> NwDbClient {

        let worker_tx = self.worker_tx();

        NwDbClient {
            worker_tx
        }
    }
}
