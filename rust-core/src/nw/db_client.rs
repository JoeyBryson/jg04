
use tokio::sync::oneshot;

use super::{NwDbClient, NwMessage, NwChat, NwContact, NwDbRequest};

use anyhow::{Context, Result};

use crate::notifications::UiEvent;

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
}


impl NwDbClient {

    fn notify(
        &self,
        event: UiEvent,
    ) {
        self.ui_listener.on_event(event);
    }

    pub async fn add_message(
        &self,
        message: NwMessage,
    ) -> Result<()> {

        let topic_id = message.topic_id.clone();

        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::AddMessage {
                message,
                reply: tx,
            },
            rx,
        ).await?;

        self.notify(
            UiEvent::ChatMessagesChanged {
                topic_id,
            }
        );

        Ok(())
    }

    pub async fn add_contact(
        &self,
        contact: NwContact,
    ) -> Result<()> {

        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::AddContact {
                contact,
                reply: tx,
            },
            rx,
        ).await?;

        self.notify(
            UiEvent::ContactsChanged
        );

        Ok(())
    }

    pub async fn add_chat(
        &self,
        chat: NwChat,
    ) -> Result<()> {

        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::AddChat {
                chat,
                reply: tx,
            },
            rx,
        ).await?;

        self.notify(
            UiEvent::ChatListChanged
        );

        Ok(())
    }

    pub async fn get_chats(&self) -> Result<Vec<NwChat>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::GetChats { reply: tx },
            rx,
        ).await
    }

    pub async fn get_chat_members(
        &self,
        topic_id: Vec<u8>,
    ) -> Result<Vec<NwContact>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::GetChatMembers { topic_id, reply: tx },
            rx,
        ).await
    }

    pub async fn get_chat(&self, topic_id: Vec<u8>) -> Result<NwChat> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::GetChat { topic_id, reply: tx },
            rx,
        ).await
    }

    pub async fn get_chat_messages(
        &self,
        topic_id: Vec<u8>,
    ) -> Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::GetChatMessages { topic_id, reply: tx },
            rx,
        ).await
    }

    pub async fn get_messages(&self) -> Result<Vec<NwMessage>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(
            NwDbRequest::GetMessages { reply: tx },
            rx,
        ).await
    }

    pub async fn add_sample_chat(&self) -> Result<()> {

        let alice = NwContact {
            name: "Alice".to_string(),
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
        let mut sent_at = 1779490800000i64;

        for i in 0..30 {
            let from_me = i % 3 == 0;

            let endpoint_id = if from_me {
                None
            } else {
                Some(if i % 2 == 0 {
                    alice.endpoint_id.clone()
                } else {
                    bob.endpoint_id.clone()
                })
            };

            let message = NwMessage {
                topic_id: topic_id.clone(),
                from_me,
                endpoint_id,
                content: format!("Sample message {}", i + 1),
                sent_at,
            };

            self.add_message(message).await?;

            sent_at += 60000;
        }

        Ok(())
    }
}
