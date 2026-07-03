use std::path::{self, PathBuf};
use tokio::sync::{mpsc, oneshot};
use rusqlite::{Connection, OpenFlags};
use anyhow::Result;

use crate::nw::NwDbRequest;
use crate::ui::UiDbRequest;

use super::{UiDbWorker, NwDbWorker};

#[derive(Debug)]
pub enum DbMode {
    ReadOnly,
    ReadWrite,
}

fn start_conn(db_path: &path::Path, mode: DbMode) -> Result<Connection> {
    let flags = match mode {
        DbMode::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
        DbMode::ReadWrite => {
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
        }
    };

    let conn = Connection::open_with_flags(db_path, flags).map_err(|e| {
        log::error!("[DB] open failed: {}", e);
        e
    })?;

    if matches!(mode, DbMode::ReadWrite) {
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");
    }

    Ok(conn)
}

fn send_reply<T>(reply: oneshot::Sender<T>, result: T) {
    let _ = reply.send(result);
}


impl UiDbWorker {

    pub fn start(worker_rx: mpsc::Receiver<UiDbRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[UI-WORKER] start db_path={:?}", db_path);

        let conn = start_conn(&db_path, DbMode::ReadOnly)?;

        log::info!("[UI-WORKER] started");

        Ok(Self { worker_rx, conn })
    }

    pub fn request_loop(mut self) {
        log::info!("[UI-WORKER] loop start");

        while let Some(request) = self.worker_rx.blocking_recv() {
            match request {
                UiDbRequest::GetChatHeaders { reply } => 
                    send_reply(reply, self.get_chats()),

                UiDbRequest::GetChatHeader { topic_id, reply } => 
                    send_reply(reply, self.get_chat(&topic_id)),

                UiDbRequest::GetChatMembers { topic_id, reply } => 
                    send_reply(reply, self.get_chat_members(&topic_id)),

                UiDbRequest::GetChatMessages { topic_id, reply } => 
                    send_reply(reply, self.get_chat_messages(&topic_id)),

                UiDbRequest::GetChatLastMessage { topic_id, reply } => 
                    send_reply(reply, self.get_chat_last_message(&topic_id)),

                UiDbRequest::GetChatData { topic_id, reply } => 
                    send_reply(reply, self.get_chat_with_messages(&topic_id)),

                UiDbRequest::GetContacts { reply } => 
                    send_reply(reply, self.get_contacts()),
            }
        }

        log::info!("[UI-WORKER] loop exit");
    }
}

impl NwDbWorker {

    pub fn start(worker_rx: mpsc::Receiver<NwDbRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[NW-WORKER] start db_path={:?}", db_path);

        let conn = start_conn(&db_path, DbMode::ReadWrite)?;

        log::info!("[NW-WORKER] ready");

        Ok(Self { worker_rx, conn })
    }

    pub fn request_loop(mut self) {
        log::info!("[NW-WORKER] loop start");

        while let Some(request) = self.worker_rx.blocking_recv() {
            match request {
                NwDbRequest::AddMessage { message, reply } => 
                    send_reply(reply, self.add_message(message)),

                NwDbRequest::AddContact { contact, reply } => 
                    send_reply(reply, self.add_contact(contact)),

                NwDbRequest::AddChat { chat, reply } => 
                    send_reply(reply, self.add_chat(chat)),

                NwDbRequest::GetChats { reply } => 
                    send_reply(reply, self.get_chats()),

                NwDbRequest::GetChatMembers { topic_id, reply } => 
                    send_reply(reply, self.get_chat_members(&topic_id)),

                NwDbRequest::GetChat { topic_id, reply } => 
                    send_reply(reply, self.get_chat(&topic_id)),

                NwDbRequest::GetChatMessages { topic_id, reply } => 
                    send_reply(reply, self.get_chat_messages(&topic_id)),

                NwDbRequest::GetMessages { reply } => 
                    send_reply(reply, self.get_messages()),
            }
        }

        log::info!("[NW-WORKER] loop exit");
    }
}