use crate::network::{NwChat, NwContact, NwMessage, SetupError, NwProfile};

use anyhow::{Result, anyhow, Context};
use std::{collections::HashMap};
use iroh::{SecretKey, PublicKey, EndpointId};
use iroh_gossip::TopicId;
use super::DbReader;

use crate::ui::{UiChatHeader, UiChatData, UiContact, UiMessage, UiSender, UiProfile};
use rusqlite::{Row, OptionalExtension}; // Added OptionalExtension


impl DbReader {

    pub fn get_nw_profile(&self) -> Result<NwProfile> {
        let mut stmt = self.conn.prepare(
            "
            SELECT secret_key, contact_name
            FROM user_profile
            ",
        )?;

        let mut rows = stmt.query([])?;
        let row = rows.next()?.ok_or(SetupError::ProfileNotSet)?;

        let secret_key =
            SecretKey::from_bytes(&row.get::<_, [u8; 32]>(0)?);

        let contact = NwContact {
            name: row.get(1)?,
            endpoint_id: EndpointId::from(secret_key.public()),
        };

        Ok(NwProfile {
            secret_key,
            contact,
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
                    name: name.clone(),
                    endpoint_id: PublicKey::from_bytes(&endpoint_id)
                    .map_err(anyhow::Error::from)
                    .with_context(|| format!("invalid public key for contact '{}'", name))?
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

    pub fn get_nw_contact(&self, endpoint_id: &EndpointId) -> Result<NwContact> {
        let endpoint_id_bytes: Vec<u8> = endpoint_id.as_bytes().to_vec();

        let (name, endpoint_id_bytes): (String, [u8; 32]) = self.conn.query_row(
            "
            SELECT contact_name, endpoint_id
            FROM contacts
            WHERE endpoint_id = ?1
            ",
            [&endpoint_id_bytes],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok(NwContact {
            name,
            endpoint_id: PublicKey::from_bytes(&endpoint_id_bytes)?,
        })
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

}