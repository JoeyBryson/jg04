use crate::network::{NwChat, NwContact, NwMessage, NwProfile};
use crate::ui::UiContact;
use anyhow::{Context, Result};
use iroh_gossip::proto::TopicId;
use super::DbWriter;



impl DbWriter {

    pub fn set_nw_profile(&self, profile: NwProfile) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO user_profile (id, secret_key, contact_name)
            VALUES (?1, ?2, ?3)
            ",
            (
                1,
                profile.secret_key.to_bytes(),
                profile.contact.name,
            ),
        )?;

        Ok(())
    }

    pub fn add_nw_message(&self, message: NwMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (topic_id, is_me, endpoint_id, content, sent_at)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                message.topic_id.as_bytes(),
                message.from_me as i32,
                message.endpoint_id.as_deref(),
                message.content,
                message.sent_at,
            ),
        )?;

        Ok(())
    }

    pub fn add_nw_contact(&self, contact: NwContact) -> Result<()> {
        self.conn.execute(
            "INSERT INTO contacts (endpoint_id, contact_name)
            VALUES (?1, ?2)",
            (
                contact.endpoint_id.as_slice(),
                contact.name
            ),
        )?;

        Ok(())
    }

    
    pub fn add_ui_contact(&self, contact: UiContact) -> Result<()> {
        let endpoint_id = contact.endpoint_id.parse()?;

        self.add_nw_contact(NwContact {
            name: contact.name,
            endpoint_id,
        })
    }

    pub fn add_nw_chat(&mut self, chat: NwChat) -> Result<()> {
        let transaction = self.conn
            .transaction()
            .context("failed to start transaction")?;

        transaction
            .execute(
                "INSERT INTO chats (topic_id, chat_name)
                VALUES (?1, ?2)",
                (
                    chat.topic_id.as_bytes(),
                    chat.name,
                ),
            )
            .context("failed to insert chat")?;

        for member in chat.members {
            transaction
                .execute(
                    "INSERT INTO chat_members (topic_id, endpoint_id)
                    VALUES (?1, ?2)",
                    (
                        chat.topic_id.as_bytes(),
                        member.endpoint_id.as_bytes(),
                    ),
                )
                .context("failed to insert chat member")?;
        }

        transaction
            .commit()
            .context("failed to commit chat transaction")?;

        Ok(())
    }

    pub fn add_chat_ui(
        &mut self,
        contacts: Vec<UiContact>,
        name: Option<String>,
    ) -> Result<String> {
        let members = contacts
            .into_iter()
            .map(|contact| -> Result<NwContact> {
                Ok(NwContact {
                    name: contact.name,
                    endpoint_id: contact.endpoint_id
                        .parse()
                        .context("failed to convert UiContact to NwContact")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let topic_id = TopicId::from_bytes(rand::random());

        self.add_nw_chat(NwChat {
            name,
            members,
            topic_id,
        })
        .context("failed to add NwChat")?;

        Ok(topic_id.to_string())
    }

    pub fn reset_database(&mut self) -> Result<()> {
        let transaction = self.conn.transaction()?;

        transaction.execute_batch(
            "
            DELETE FROM user_profile;
            DELETE FROM contacts;
            ",
        )?;

        transaction.commit()?;

        Ok(())
    }
}
