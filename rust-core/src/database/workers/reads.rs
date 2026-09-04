use crate::network::{NwChat, NwContact, NwMessage, NwProfile, SetupError};

use anyhow::{Result, anyhow};
use std::{collections::HashMap};
use iroh::{SecretKey, PublicKey};
use iroh_gossip::TopicId;
use super::DbReader;

use crate::ui::{UiChatHeader, UiChatData, UiContact, UiMessage, UiSender};
use rusqlite::{Row, OptionalExtension}; // Added OptionalExtension


impl DbReader {

    pub fn get_nw_profile(&self) -> Result<NwProfile> {
        let mut stmt = self.conn.prepare(
            "
            SELECT secret_key
            FROM user_profile
            ",
        )?;

        let mut rows = stmt.query([])?;
        let row = rows.next()?.ok_or(SetupError::ProfileNotSet)?;

        let secret_key =
            SecretKey::from_bytes(&row.get::<_, [u8; 32]>(0)?);

        Ok(NwProfile {
            secret_key
        })
    }

    pub fn get_nw_chats(&self) -> Result<Vec<NwChat>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                c.topic_id,
                c.chat_name,
                con.contact_name,
                con.endpoint_id
            FROM chats c
            LEFT JOIN chat_members cm
                ON c.topic_id = cm.topic_id
            LEFT JOIN contacts con
                ON cm.endpoint_id = con.endpoint_id
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, [u8;32]>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<[u8;32]>>(3)?,
            ))
        })?;

        let mut chats: HashMap<[u8;32], NwChat> = HashMap::new();

        for row in rows {
            let (topic_id_bytes, chat_name, contact_name, endpoint_id) = row?;

            let chat = chats.entry(topic_id_bytes).or_insert_with(|| NwChat {
                name: chat_name,
                members: Vec::new(),
                topic_id: TopicId::from_bytes(topic_id_bytes),
            });

            if let (Some(name), Some(endpoint_id)) = (contact_name, endpoint_id) {
                chat.members.push(NwContact {
                    name,
                    endpoint_id: PublicKey::from_bytes(&endpoint_id)?,
                });
            }
        }

        for chat in chats.values() {
            if chat.members.is_empty() {
                return Err(anyhow!(
                    "Chat {:?} has no members",
                    chat.topic_id
                ));
            }
        }

        Ok(chats.into_values().collect())
    }

    pub fn get_nw_chat_members(&self, topic_id: &[u8]) -> Result<Vec<NwContact>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT c.contact_name, c.endpoint_id
            FROM chat_members cm
            LEFT JOIN contacts c
                ON c.endpoint_id = cm.endpoint_id
            WHERE cm.topic_id = ?1
            ",
        )?;

        let mut rows = stmt.query([topic_id])?;
        let mut members = Vec::new();

        while let Some(row) = rows.next()? {
            let endpoint_id = row.get(1)?;

            members.push(NwContact {
                name: row.get(0)?,
                endpoint_id: PublicKey::from_bytes(&endpoint_id)?,
            });
        }

        if members.is_empty() {
            return Err(anyhow!("chat not found or has no members"));
        }

        Ok(members)
    }

    pub fn get_nw_chat(&self, topic_id: &[u8]) -> Result<NwChat> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                c.topic_id,
                c.chat_name,
                con.contact_name,
                con.endpoint_id
            FROM chats c
            LEFT JOIN chat_members cm
                ON c.topic_id = cm.topic_id
            LEFT JOIN contacts con
                ON cm.endpoint_id = con.endpoint_id
            WHERE c.topic_id = ?1
            ",
        )?;

        let mut rows = stmt.query([topic_id])?;
        let mut chat: Option<NwChat> = None;

        while let Some(row) = rows.next()? {
            let row_topic_id_bytes: [u8; 32] = row.get(0)?;
            let row_chat_name: Option<String> = row.get(1)?;

            let chat_ref = chat.get_or_insert_with(|| NwChat {
                topic_id: TopicId::from_bytes(row_topic_id_bytes),
                name: row_chat_name,
                members: Vec::new(),
            });

            let optional_name: Option<String> = row.get(2)?;
            let optional_endpoint_id_bytes: Option<[u8;32]> = row.get(3)?;

            if let (Some(name), Some(endpoint_id_bytes)) =
                (optional_name, optional_endpoint_id_bytes)
            {
                chat_ref.members.push(NwContact {
                    name,
                    endpoint_id: PublicKey::from_bytes(&endpoint_id_bytes)?,
                });
            }
        }

        let chat = chat.ok_or_else(|| anyhow!("chat not found"))?;

        if chat.members.is_empty() {
            return Err(anyhow!("chat has no members"));
        }

        Ok(chat)
    }

    pub fn get_nw_chat_messages(&self, topic_id: &[u8]) -> Result<Vec<NwMessage>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT topic_id, is_me, endpoint_id, content, sent_at
            FROM messages
            WHERE topic_id = ?1
            ",
        )?;

        let mut rows = stmt.query([topic_id])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            let endpoint_id = row
                .get::<_, Option<[u8; 32]>>(2)?
                .map(|bytes| PublicKey::from_bytes(&bytes))
                .transpose()?;

            messages.push(NwMessage {
                topic_id: TopicId::from_bytes(row.get(0)?),
                from_me: row.get::<_, i32>(1)? != 0,
                endpoint_id,
                content: row.get(3)?,
                sent_at: row.get(4)?,
            });
        }

        Ok(messages)
    }

    pub fn get_nw_messages(&self) -> Result<Vec<NwMessage>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT topic_id, is_me, endpoint_id, content, sent_at
            FROM messages
            ",
        )?;

        let mut rows = stmt.query([])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            let endpoint_id = row
                .get::<_, Option<[u8; 32]>>(2)?
                .map(|bytes| PublicKey::from_bytes(&bytes))
                .transpose()?;

            messages.push(NwMessage {
                topic_id: TopicId::from_bytes(row.get(0)?),
                from_me: row.get::<_, i32>(1)? != 0,
                endpoint_id,
                content: row.get(3)?,
                sent_at: row.get(4)?,
            });
        }

        Ok(messages)
    }

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
            sent_at
        })
    }

    pub fn profile_exists(&self) -> Result<bool> {
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM user_profile WHERE id = 1)",
            [],
            |row| row.get(0),
        )?;

        Ok(exists)
    }

    pub fn get_ui_last_chat_message(
        &self,
        topic_id: &str
    ) -> Result<Option<UiMessage>> { // Updated return type
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

    pub fn get_ui_chat_members(
        &self,
        topic_id: &str
    ) -> Result<Vec<UiContact>> {
        let topic_id = Self::decode_hex_id(topic_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT c.contact_name, c.endpoint_id
             FROM chat_members cm
             JOIN contacts c ON cm.endpoint_id = c.endpoint_id
             WHERE cm.topic_id = ?1",
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

    pub fn get_ui_chat_header(
        &self,
        topic_id: &str
    ) -> Result<UiChatHeader> {
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
        let mut stmt = self.conn.prepare("SELECT topic_id FROM chats")?;

        let topic_ids = stmt
            .query_map([], |row| row.get::<_, Vec<u8>>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        topic_ids
            .into_iter()
            .map(|topic_id| {
                self.get_ui_chat_header(&hex::encode(topic_id))
            })
            .collect()
    }

    pub fn get_ui_chat_messages(
        &self,
        topic_id: &str
    ) -> Result<Vec<UiMessage>> {
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

    pub fn get_ui_chat_data(
        &self,
        topic_id: &str
    ) -> Result<UiChatData> {
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
}