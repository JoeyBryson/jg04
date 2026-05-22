use rusqlite::Connection;
use std::fs;
use std::path;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use crate::nw;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

const SCHEMA: &str = include_str!("../sql/schema.sql");
pub enum Command {
    AddMessage {
        message: nw::Message,
        reply: oneshot::Sender<Result<()>>,
    },
    AddContact {
        contact: nw::Contact,
        reply: oneshot::Sender<Result<()>>,
    },
    AddChat {
        chat: nw::Chat,
        reply: oneshot::Sender<Result<()>>,
    },
    GetChats {
        reply: oneshot::Sender<Result<Vec<nw::Chat>>>,
    },
    GetChatMembers {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<nw::Contact>>>,
    },
    GetChat {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<nw::Chat>>,
    },
    GetChatMessages {
        topic_id: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<nw::Message>>>,
    },
    GetMessages {
        reply: oneshot::Sender<Result<Vec<nw::Message>>>,
    },
}

struct ConnectionHandle {
    conn: Connection
}


pub fn worker<P: AsRef<path::Path>>(
    mut db_rx: mpsc::Receiver<Command>,
    path: P
) -> Result<()>{

    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    log::info!("db worker started db path:{:?}", path.as_ref());
    let mut handle = ConnectionHandle{conn: Connection::open(path)?};
    log::info!("connection openned");
    handle.conn.execute_batch(SCHEMA)?;
    log::info!("schema executed");

    while let Some(command) = db_rx.blocking_recv() {
        match command {
            Command::AddMessage { message, reply } => {
                let result = handle.add_message(message);
                let _ = reply.send(result);
            }

            Command::AddContact { contact, reply } => {
                let result = handle.add_contact(contact);
                let _ = reply.send(result);
            }

            Command::AddChat { chat, reply } => {
                let result = handle.add_chat(chat);
                let _ = reply.send(result);
            }

            Command::GetChats { reply } => {
                let result = handle.get_chats();
                let _ = reply.send(result);
            }

            Command::GetChatMembers { topic_id, reply } => {
                let result = handle.get_chat_members(&topic_id);
                let _ = reply.send(result);
            }

            Command::GetChat { topic_id, reply } => {
                let result = handle.get_chat(&topic_id);
                let _ = reply.send(result);
            }

            Command::GetChatMessages { topic_id, reply } => {
                let result = handle.get_chat_messages(&topic_id);
                let _ = reply.send(result);
            }

            Command::GetMessages { reply } => {
                let result = handle.get_messages();
                let _ = reply.send(result);
            }
        }
    }
    log::error!("DB Worker Function Exited?");
    Ok(())
}

impl ConnectionHandle {
    fn add_message(&self, message: nw::Message) -> Result<()>{

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


    fn add_contact(&self, contact: nw::Contact) -> Result<()>{
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


    fn add_chat(&mut self, chat: nw::Chat) -> Result<()> {

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


    fn get_chats(&self) -> Result<Vec<nw::Chat>> {

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

        let mut chats: HashMap<Vec<u8>, nw::Chat> = HashMap::new();

        for row in rows {
            let (topic_id, chat_name, contact_name, endpoint_id) = row?;

            let chat = chats.entry(topic_id.clone()).or_insert_with(|| nw::Chat {
                topic_id,
                name: chat_name,
                members: Vec::new(),
            });

            if let (Some(name), Some(endpoint_id)) = (contact_name, endpoint_id) {
                chat.members.push(nw::Contact { 
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

    fn get_chat_members(&self, topic_id: &[u8]) -> Result<Vec<nw::Contact>> {

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
            members.push(nw::Contact {
                name: row.get(0)?,
                endpoint_id: row.get(1)?,
            });
        }

        if members.is_empty() {
            return Err(anyhow!("chat not found or has no members"));
        }

        Ok(members)
    }

    fn get_chat(&self, topic_id: &[u8]) -> Result<nw::Chat> {

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

        let mut chat: Option<nw::Chat> = None;

        while let Some(row) = rows.next()? {

            let row_topic_id: Vec<u8> = row.get(0)?;
            let row_chat_name: Option<String> = row.get(1)?;

            let chat_ref = chat.get_or_insert_with(|| nw::Chat {
                topic_id: row_topic_id,
                name: row_chat_name,
                members: Vec::new(),
            });

            let contact_name: Option<String> = row.get(2)?;
            let endpoint_id: Option<Vec<u8>> = row.get(3)?;

            if let (Some(name), Some(endpoint_id)) = (contact_name, endpoint_id) {
                chat_ref.members.push(nw::Contact {
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

    fn get_chat_messages(&self, topic_id: &[u8]) -> Result<Vec<nw::Message>> {

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
            messages.push(nw::Message {
                topic_id: row.get(0)?,
                from_me: row.get::<_, i32>(1)? != 0,
                endpoint_id: row.get(2)?,
                content: row.get(3)?,
                sent_at: row.get(4)?,
            });
        }

        Ok(messages)
    }

    fn get_messages(&self) -> Result<Vec<nw::Message>> {

        let mut stmt = self.conn.prepare(
            "
            SELECT topic_id, is_me, endpoint_id, content, sent_at
            FROM messages
            ",
        )?;

        let mut rows = stmt.query([])?;

        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            messages.push(nw::Message {
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