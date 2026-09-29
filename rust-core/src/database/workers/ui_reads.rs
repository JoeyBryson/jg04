//! Database read functions for Ui Module types
//! For database design read ../sql/schema.sql
//! Quick function summary (inputs -> output):
//! - `get_ui_last_chat_message(topic_id: &str) -> Result<Option<UiMessage>>`
//! - `get_ui_chat_members(topic_id: &str) -> Result<Vec<UiContact>>`
//! - `get_ui_chat_header(topic_id: &str) -> Result<UiChatHeader>`
//! - `get_ui_chat_headers() -> Result<Vec<UiChatHeader>>`
//! - `get_ui_chat_messages(topic_id: &str) -> Result<Vec<UiMessage>>`
//! - `get_ui_chat_data(topic_id: &str) -> Result<UiChatData>`
//! - `get_ui_contacts() -> Result<Vec<UiContact>>`
//! - `get_ui_profile() -> Result<Option<UiProfile>>`

use super::DbReader;
use anyhow::Result;
use anyhow::{Context};

use crate::ui::{UiChatData, UiChatHeader, UiContact, UiMessage, UiProfile, UiSender};
use rusqlite::{OptionalExtension, Row};

impl DbReader {
    fn map_message_row(
        row: &Row,
        my_endpoint_id: &[u8],
    ) -> rusqlite::Result<UiMessage> {
        let endpoint_id: Vec<u8> = row.get(0)?;
        let contact_name: Option<String> = row.get(1)?;
        let content: String = row.get(2)?;
        let sent_at: i64 = row.get(3)?;

        let sender = if endpoint_id == my_endpoint_id {
            UiSender::Me
        } else {
            UiSender::Other(UiContact {
                name: contact_name.unwrap_or_else(|| "Unknown".to_string()),
                endpoint_id: hex::encode(endpoint_id),
            })
        };

        Ok(UiMessage {
            sender,
            content,
            sent_at,
        })
    }

    fn get_my_endpoint_id(&self) -> Result<Vec<u8>> {
        self.conn
            .query_row(
                "SELECT endpoint_id FROM user_profile WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .context("Failed to get my endpoint ID: user profile has not been initialized")
    }

    pub fn get_ui_last_chat_message(&self, topic_id: &str) -> Result<Option<UiMessage>> {
        let topic_id = hex::decode(topic_id)?;
        let my_endpoint_id = self.get_my_endpoint_id()?;

        let mut stmt = self.conn.prepare(
            "SELECT m.endpoint_id, c.contact_name, m.content, m.sent_at
             FROM messages m
             LEFT JOIN contacts c ON m.endpoint_id = c.endpoint_id
             WHERE m.topic_id = ?1
             ORDER BY m.sent_at DESC, m.id DESC
             LIMIT 1",
        )?;

        let msg = stmt
            .query_row([topic_id], |row| {
                Self::map_message_row(row, &my_endpoint_id)
            })
            .optional()?;

        Ok(msg)
    }

    pub fn get_ui_chat_members(&self, topic_id: &str) -> Result<Vec<UiContact>> {
        let topic_id = hex::decode(topic_id)?;

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
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(members)
    }

    pub fn get_ui_chat_header(&self, topic_id: &str) -> Result<UiChatHeader> {
        let topic_id_bytes = hex::decode(topic_id)?;

        let name: Option<String> = self.conn.query_row(
            "SELECT chat_name
             FROM chats
             WHERE topic_id = ?1",
            [&topic_id_bytes],
            |row| row.get(0),
        )?;

        Ok(UiChatHeader {
            name,
            members: self.get_ui_chat_members(topic_id)?,
            topic_id: topic_id.to_string(),
            last_message: self.get_ui_last_chat_message(topic_id)?,
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
        let topic_id = hex::decode(topic_id)?;
        let my_endpoint_id = self.get_my_endpoint_id()?;

        let mut stmt = self.conn.prepare(
            "SELECT m.endpoint_id, c.contact_name, m.content, m.sent_at
             FROM messages m
             LEFT JOIN contacts c ON m.endpoint_id = c.endpoint_id
             WHERE m.topic_id = ?1
             ORDER BY m.sent_at ASC, m.id ASC",
        )?;

        let rows = stmt.query_map([topic_id], |row| {
            Self::map_message_row(row, &my_endpoint_id)
        })?;

        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
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
            "SELECT contact_name, endpoint_id
             FROM user_profile
             WHERE id = 1",
        )?;

        let profile = stmt
            .query_row([], |row| {
                let name: String = row.get(0)?;
                let endpoint_id: Vec<u8> = row.get(1)?;

                Ok(UiProfile {
                    name,
                    endpoint_id: hex::encode(endpoint_id),
                })
            })
            .optional()?;

        Ok(profile)
    }
}