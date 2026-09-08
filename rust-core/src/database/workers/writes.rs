use crate::network::{NwChat, NwContact, NwMessage, NwProfile};

use anyhow::{Result};
use super::DbWriter;



impl DbWriter {

    pub fn set_nw_profile(&self, profile: NwProfile) -> Result<()> {
        // if self.get_nw_profile().is_ok() {
        //     return Err(SetupError::ProfileAlreadySet.into());
        // }

        self.conn.execute(
            "INSERT INTO user_profile (id, secret_key)
        VALUES (?1, ?2)",
            (
                1,
                profile.secret_key.to_bytes()
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

    pub fn add_nw_chat(&mut self, chat: NwChat) -> Result<()> {
        let transaction = self.conn.transaction()?;

        transaction.execute(
            "INSERT INTO chats (topic_id, chat_name)
            VALUES (?1, ?2)",
            (
                chat.topic_id.as_bytes(),
                chat.name,
            ),
        )?;

        for member in chat.members {
            transaction.execute(
                "INSERT INTO chat_members (topic_id, endpoint_id)
                VALUES (?1, ?2)",
                (
                    chat.topic_id.as_bytes(),
                    member.endpoint_id.as_bytes(),
                ),
            )?;
        }

        transaction.commit()?;

        Ok(())
    }
}

