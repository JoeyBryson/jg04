use super::DbReader;
use anyhow::Result;
use iroh::SecretKey;

use crate::ui::{UiChatData, UiChatHeader, UiContact, UiMessage, UiProfile, UiSender};
use rusqlite::{OptionalExtension, Row}; // Added OptionalExtension

impl DbReader {
    fn decode_hex_id(id: &str) -> Result<Vec<u8>> {
        Ok(hex::decode(id)?)
    }

    fn map_message_row(row: &Row) -> rusqlite::Result<UiMessage> {
        let is_me: i64 = row.get(0)?;
        let endpoint_id: Option<Vec<u8>> = row.get(1)?;
        let contact_name: Option<String> = row.get(2)?;
        let content: String = row.get(3)?;
        let sent_at: i64 = row.get(4)?;

        let sender = if is_me != 0 {
            UiSender::Me
        } else {
            UiSender::Other(UiContact {
                name: contact_name.unwrap_or_else(|| "Unknown".to_string()),
                endpoint_id: hex::encode(endpoint_id.unwrap_or_default()),
            })
        };

        Ok(UiMessage {
            sender,
            content,
            sent_at,
        })
    }

    pub fn get_ui_last_chat_message(&self, topic_id: &str) -> Result<Option<UiMessage>> {
        // Updated return type
        let topic_id = Self::decode_hex_id(topic_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT m.is_me, m.endpoint_id, c.contact_name, m.content, m.sent_at
             FROM messages m
             LEFT JOIN contacts c ON m.endpoint_id = c.endpoint_id
             WHERE m.topic_id = ?1
             ORDER BY m.sent_at DESC, m.id DESC
             LIMIT 1",
        )?;

        // .optional() converts a QueryReturnedNoRows error into Ok(None)
        let msg = stmt
            .query_row([topic_id], Self::map_message_row)
            .optional()?;

        Ok(msg)
    }

    pub fn get_ui_chat_members(&self, topic_id: &str) -> Result<Vec<UiContact>> {
        let topic_id = Self::decode_hex_id(topic_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT c.contact_name, c.endpoint_id
             FROM chat_members cm
             JOIN contacts c ON cm.endpoint_id = c.endpoint_id
             WHERE cm.topic_id = ?1
             ORDER BY c.contact_name, cm.endpoint_id",
        )?;

        let members = stmt
            .query_map([topic_id], |row| {
                let endpoint_id: Vec<u8> = row.get(1)?;
                Ok(UiContact {
                    name: row.get(0)?,
                    endpoint_id: hex::encode(endpoint_id),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(members)
    }

    pub fn get_ui_chat_header(&self, topic_id: &str) -> Result<UiChatHeader> {
        let topic_id_bytes = Self::decode_hex_id(topic_id)?;

        let name: Option<String> = self.conn.query_row(
            "SELECT chat_name FROM chats WHERE topic_id = ?1",
            [&topic_id_bytes],
            |row| row.get(0),
        )?;

        Ok(UiChatHeader {
            name,
            members: self.get_ui_chat_members(topic_id)?,
            topic_id: topic_id.to_string(),
            last_message: self.get_ui_last_chat_message(topic_id)?, // Now expects Option<UiMessage>
        })
    }

    pub fn get_ui_chat_headers(&self) -> Result<Vec<UiChatHeader>> {
        let mut stmt = self
            .conn
            .prepare("SELECT topic_id FROM chats ORDER BY topic_id")?;

        let topic_ids = stmt
            .query_map([], |row| row.get::<_, Vec<u8>>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        topic_ids
            .into_iter()
            .map(|topic_id| self.get_ui_chat_header(&hex::encode(topic_id)))
            .collect()
    }

    pub fn get_ui_chat_messages(&self, topic_id: &str) -> Result<Vec<UiMessage>> {
        let topic_id = Self::decode_hex_id(topic_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT m.is_me, m.endpoint_id, c.contact_name, m.content, m.sent_at
             FROM messages m
             LEFT JOIN contacts c ON m.endpoint_id = c.endpoint_id
             WHERE m.topic_id = ?1
             ORDER BY m.sent_at ASC, m.id ASC",
        )?;

        let rows = stmt.query_map([topic_id], Self::map_message_row)?;

        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn get_ui_chat_data(&self, topic_id: &str) -> Result<UiChatData> {
        Ok(UiChatData {
            chat: self.get_ui_chat_header(topic_id)?,
            messages: self.get_ui_chat_messages(topic_id)?,
        })
    }

    pub fn get_ui_contacts(&self) -> Result<Vec<UiContact>> {
        let mut stmt = self.conn.prepare(
            "SELECT contact_name, endpoint_id
             FROM contacts
             ORDER BY contact_name",
        )?;

        let contacts = stmt
            .query_map([], |row| {
                let endpoint_id: Vec<u8> = row.get(1)?;
                Ok(UiContact {
                    name: row.get(0)?,
                    endpoint_id: hex::encode(endpoint_id),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(contacts)
    }

    pub fn get_ui_profile(&self) -> Result<Option<UiProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT secret_key, contact_name
            FROM user_profile",
        )?;

        let mut rows = stmt.query([])?;

        let profile = match rows.next()? {
            Some(row) => {
                let secret_key_bytes: [u8; 32] = row.get(0)?;
                let name: String = row.get(1)?;

                let secret_key = SecretKey::from_bytes(&secret_key_bytes);
                let public_key = secret_key.public();

                Some(UiProfile {
                    name,
                    endpoint_id: hex::encode(public_key.as_bytes()),
                })
            }
            None => None,
        };

        Ok(profile)
    }
}
