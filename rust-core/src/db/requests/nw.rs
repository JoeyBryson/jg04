use crate::nw::{NwMessage, NwContact, NwChat};

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use super::super::NwDbWorker;

impl NwDbWorker {
    pub fn add_message(&self, message: NwMessage) -> Result<()>{

        self.conn.execute(
            "INSERT INTO messages (topic_id, is_me, endpoint_id, content, sent_at)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                message.topic_id.as_slice(),
                message.from_me as i32,
                message.endpoint_id.as_deref(), 
                message.content,
                message.sent_at,
            ),
        )?;

        Ok(())
    }


    pub fn add_contact(&self, contact: NwContact) -> Result<()>{
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


    pub fn add_chat(&mut self, chat: NwChat) -> Result<()> {

        let transaction = self.conn.transaction()?;

        transaction.execute(
            "INSERT INTO chats (topic_id, chat_name)
            VALUES (?1, ?2)",
            (
                chat.topic_id.as_slice(),
                chat.name,
            ),
        )?;

        for member in chat.members {
            transaction.execute(
                "INSERT INTO chat_members (topic_id, endpoint_id)
                VALUES (?1, ?2)",
                (
                    chat.topic_id.as_slice(),
                    member.endpoint_id.as_slice(),
                ),
            )?;
        }

        transaction.commit()?;

        Ok(())
    }


    pub fn get_chats(&self) -> Result<Vec<NwChat>> {

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
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<Vec<u8>>>(3)?,
            ))
        })?;

        let mut chats: HashMap<Vec<u8>, NwChat> = HashMap::new();

        for row in rows {
            let (topic_id, chat_name, contact_name, endpoint_id) = row?;

            let chat = chats.entry(topic_id.clone()).or_insert_with(|| NwChat {
                topic_id,
                name: chat_name,
                members: Vec::new(),
            });

            if let (Some(name), Some(endpoint_id)) = (contact_name, endpoint_id) {
                chat.members.push(NwContact { 
                    name, 
                    endpoint_id 
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

    pub fn get_chat_members(&self, topic_id: &[u8]) -> Result<Vec<NwContact>> {

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
            members.push(NwContact {
                name: row.get(0)?,
                endpoint_id: row.get(1)?,
            });
        }

        if members.is_empty() {
            return Err(anyhow!("chat not found or has no members"));
        }

        Ok(members)
    }

    pub fn get_chat(&self, topic_id: &[u8]) -> Result<NwChat> {

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

            let row_topic_id: Vec<u8> = row.get(0)?;
            let row_chat_name: Option<String> = row.get(1)?;

            let chat_ref = chat.get_or_insert_with(|| NwChat {
                topic_id: row_topic_id,
                name: row_chat_name,
                members: Vec::new(),
            });

            let contact_name: Option<String> = row.get(2)?;
            let endpoint_id: Option<Vec<u8>> = row.get(3)?;

            if let (Some(name), Some(endpoint_id)) = (contact_name, endpoint_id) {
                chat_ref.members.push(NwContact {
                    name,
                    endpoint_id,
                });
            }
        }

        let chat = chat.ok_or_else(|| anyhow!("chat not found"))?;

        if chat.members.is_empty() {
            return Err(anyhow!("chat has no members"));
        }

        Ok(chat)
    }

    pub fn get_chat_messages(&self, topic_id: &[u8]) -> Result<Vec<NwMessage>> {

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
            messages.push(NwMessage {
                topic_id: row.get(0)?,
                from_me: row.get::<_, i32>(1)? != 0,
                endpoint_id: row.get(2)?,
                content: row.get(3)?,
                sent_at: row.get(4)?,
            });
        }

        Ok(messages)
    }

    pub fn get_messages(&self) -> Result<Vec<NwMessage>> {

        let mut stmt = self.conn.prepare(
            "
            SELECT topic_id, is_me, endpoint_id, content, sent_at
            FROM messages
            ",
        )?;

        let mut rows = stmt.query([])?;

        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            messages.push(NwMessage {
                topic_id: row.get(0)?,
                from_me: row.get::<_, i32>(1)? != 0,
                endpoint_id: row.get(2)?,
                content: row.get(3)?,
                sent_at: row.get(4)?,
            });
        }

        Ok(messages)
    }
}