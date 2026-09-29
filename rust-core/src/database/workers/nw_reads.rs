//! Database read functions for network types (crate defined Nw types and Iroh types)
//! For database design read ../sql/schema.sql
//! Quick function summary (inputs -> output):
//! - `get_nw_profile() -> Result<NwProfile>`
//! - `get_nw_chats() -> Result<Vec<NwChat>>`
//! - `get_nw_contact(endpoint_id: &EndpointId) -> Result<NwContact>`
//! - `get_nw_chat_members(topic_id: &TopicId) -> Result<Vec<NwChatMember>>`
//! - `get_nw_chat(topic_id: &TopicId) -> Result<NwChat>`
//! - `get_nw_chat_messages(topic_id: &TopicId) -> Result<Vec<NwMessage>>`
//! - `get_nw_messages() -> Result<Vec<NwMessage>>`

use crate::network::{
    NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile,
};
use super::DbReader;
use anyhow::{Context, Result, anyhow};
use iroh::{EndpointId, PublicKey, SecretKey};
use iroh_gossip::TopicId;
use std::collections::BTreeMap;


impl DbReader {
    pub fn get_nw_profile(&self) -> Result<NwProfile> {
        let mut stmt = self.conn.prepare(
            "
            SELECT endpoint_id, secret_key, contact_name
            FROM user_profile
            ",
        )?;

        let mut rows = stmt.query([])?;
        let row = rows.next()?.ok_or(anyhow!("Profile not set"))?;

        let endpoint_id = PublicKey::from_bytes(&row.get::<_, [u8; 32]>(0)?)?;
        let secret_key = SecretKey::from_bytes(&row.get::<_, [u8; 32]>(1)?);

        let contact = NwContact {
            name: row.get(2)?,
            endpoint_id: EndpointId::from(endpoint_id),
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
                con.endpoint_id,
                cm.status
            FROM chats c
            JOIN chat_members cm
                ON c.topic_id = cm.topic_id
            JOIN contacts con
                ON cm.endpoint_id = con.endpoint_id
            ORDER BY c.topic_id, cm.endpoint_id
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, [u8; 32]>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, [u8; 32]>(3)?,
                row.get::<_, i32>(4)?,
            ))
        })?;

        let mut chats: BTreeMap<[u8; 32], NwChat> = BTreeMap::new();

        for row in rows {
            let (topic_id_bytes, chat_name, contact_name, endpoint_id, status) = row?;

            let chat = chats.entry(topic_id_bytes).or_insert_with(|| NwChat {
                name: chat_name,
                members: Vec::new(),
                topic_id: TopicId::from_bytes(topic_id_bytes),
            });

            chat.members.push(NwChatMember {
                contact: NwContact {
                    name: contact_name,
                    endpoint_id: PublicKey::from_bytes(&endpoint_id)
                        .map_err(anyhow::Error::from)
                        .with_context(|| "invalid public key for chat member")?,
                },
                status: NwChatMemberStatus::try_from(status)
                    .map_err(anyhow::Error::msg)?,
            });
        }

        if chats.values().any(|chat| chat.members.is_empty()) {
            return Err(anyhow!("Chat has no members"));
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

    pub fn get_nw_chat_members(&self, topic_id: &TopicId) -> Result<Vec<NwChatMember>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT c.contact_name, c.endpoint_id, cm.status
            FROM chat_members cm
            JOIN contacts c
                ON c.endpoint_id = cm.endpoint_id
            WHERE cm.topic_id = ?1
            ORDER BY cm.endpoint_id
            ",
        )?;

        let mut rows = stmt.query([topic_id.as_bytes()])?;
        let mut members = Vec::new();

        while let Some(row) = rows.next()? {
            let endpoint_id: [u8; 32] = row.get(1)?;
            let status: i32 = row.get(2)?;

            members.push(NwChatMember {
                contact: NwContact {
                    name: row.get(0)?,
                    endpoint_id: PublicKey::from_bytes(&endpoint_id)?,
                },
                status: NwChatMemberStatus::try_from(status)
                    .map_err(anyhow::Error::msg)?,
            });
        }

        if members.is_empty() {
            return Err(anyhow!("chat not found or has no members"));
        }

        Ok(members)
    }

    pub fn get_nw_chat(&self, topic_id: &TopicId) -> Result<NwChat> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                c.topic_id,
                c.chat_name,
                con.contact_name,
                con.endpoint_id,
                cm.status
            FROM chats c
            JOIN chat_members cm
                ON c.topic_id = cm.topic_id
            JOIN contacts con
                ON cm.endpoint_id = con.endpoint_id
            WHERE c.topic_id = ?1
            ORDER BY cm.endpoint_id
            ",
        )?;

        let mut rows = stmt.query([topic_id.as_bytes()])?;
        let mut chat: Option<NwChat> = None;

        while let Some(row) = rows.next()? {
            let row_topic_id_bytes: [u8; 32] = row.get(0)?;
            let row_chat_name: Option<String> = row.get(1)?;

            let chat_ref = chat.get_or_insert_with(|| NwChat {
                topic_id: TopicId::from_bytes(row_topic_id_bytes),
                name: row_chat_name,
                members: Vec::new(),
            });

            let name: String = row.get(2)?;
            let endpoint_id_bytes: [u8; 32] = row.get(3)?;
            let status: i32 = row.get(4)?;

            chat_ref.members.push(NwChatMember {
                contact: NwContact {
                    name,
                    endpoint_id: PublicKey::from_bytes(&endpoint_id_bytes)?,
                },
                status: NwChatMemberStatus::try_from(status)
                    .map_err(anyhow::Error::msg)?,
            });
        }

        let chat = chat.ok_or_else(|| anyhow!("chat not found"))?;

        if chat.members.is_empty() {
            return Err(anyhow!("chat has no members"));
        }

        Ok(chat)
    }

    pub fn get_nw_chat_messages(&self, topic_id: &TopicId) -> Result<Vec<NwMessage>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT topic_id, endpoint_id, content, sent_at
            FROM messages
            WHERE topic_id = ?1
            ORDER BY id ASC
            ",
        )?;

        let mut rows = stmt.query([topic_id.as_bytes()])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            let endpoint_id =
                PublicKey::from_bytes(&row.get::<_, [u8; 32]>(1)?)?;

            messages.push(NwMessage {
                topic_id: TopicId::from_bytes(row.get(0)?),
                endpoint_id,
                content: row.get(2)?,
                sent_at: row.get(3)?,
            });
        }

        Ok(messages)
    }

    pub fn get_nw_messages(&self) -> Result<Vec<NwMessage>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT topic_id, endpoint_id, content, sent_at
            FROM messages
            ORDER BY id ASC
            ",
        )?;

        let mut rows = stmt.query([])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            let endpoint_id =
                PublicKey::from_bytes(&row.get::<_, [u8; 32]>(1)?)?;

            messages.push(NwMessage {
                topic_id: TopicId::from_bytes(row.get(0)?),
                endpoint_id,
                content: row.get(2)?,
                sent_at: row.get(3)?,
            });
        }

        Ok(messages)
    }
}
