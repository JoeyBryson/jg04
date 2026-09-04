use std::path::{self, PathBuf};
use tokio::sync::mpsc;
use rusqlite::{Connection, OpenFlags};
use anyhow::Result;

mod reads;
mod writes;
use super::requests::{ReadRequest, WriteRequest};

pub struct DbReader{
    rx: mpsc::Receiver<ReadRequest>,
    conn: rusqlite::Connection
}

pub struct DbWriter{
    rx: mpsc::Receiver<WriteRequest>,
    conn: rusqlite::Connection
}


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

macro_rules! dispatch {
    ($reply:expr, $expr:expr) => {{
        let _ = $reply.send($expr);
    }};
}

impl DbReader {
    pub fn start(worker_rx: mpsc::Receiver<ReadRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[DB-READER] start db_path={:?}", db_path);
        let conn = start_conn(&db_path, DbMode::ReadOnly)?;
        log::info!("[DB-READER] started");
        Ok(Self { rx: worker_rx, conn })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-READER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            match request {
                ReadRequest::ProfileExists { reply } => dispatch!(reply, self.profile_exists()),
                ReadRequest::GetUiChatHeaders { reply } => dispatch!(reply, self.get_ui_chat_headers()),
                ReadRequest::GetUiChatHeader { topic_id, reply } => dispatch!(reply, self.get_ui_chat_header(&topic_id)),
                ReadRequest::GetUiChatMembers { topic_id, reply } => dispatch!(reply, self.get_ui_chat_members(&topic_id)),
                ReadRequest::GetUiChatMessages { topic_id, reply } => dispatch!(reply, self.get_ui_chat_messages(&topic_id)),
                ReadRequest::GetUiChatLastMessage { topic_id, reply } => dispatch!(reply, self.get_ui_last_chat_message(&topic_id)),
                ReadRequest::GetUiChatData { topic_id, reply } => dispatch!(reply, self.get_ui_chat_data(&topic_id)),
                ReadRequest::GetUiContacts { reply } => dispatch!(reply, self.get_ui_contacts()),
                ReadRequest::GetNwProfile { reply } => dispatch!(reply, self.get_nw_profile()),
                ReadRequest::GetNwChats { reply } => dispatch!(reply, self.get_nw_chats()),
                ReadRequest::GetNwChatMembers { topic_id, reply } => dispatch!(reply, self.get_nw_chat_members(&topic_id)),
                ReadRequest::GetNwChat { topic_id, reply } => dispatch!(reply, self.get_nw_chat(&topic_id)),
                ReadRequest::GetNwChatMessages { topic_id, reply } => dispatch!(reply, self.get_nw_chat_messages(&topic_id)),
                ReadRequest::GetNwMessages { reply } => dispatch!(reply, self.get_nw_messages()),
            }
        }

        log::info!("[DB-READER] loop exit");
    }
}

impl DbWriter {
    pub fn start(worker_rx: mpsc::Receiver<WriteRequest>, db_path: PathBuf) -> Result<Self> {
        log::info!("[DB-WRITER] start db_path={:?}", db_path);
        let conn = start_conn(&db_path, DbMode::ReadWrite)?;
        log::info!("[DB-WRITER] ready");
        Ok(Self { rx: worker_rx, conn })
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-WRITER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            match request {
                WriteRequest::SetNwProfile { profile, reply } => dispatch!(reply, self.set_nw_profile(profile)),
                WriteRequest::AddNwMessage { message, reply } => dispatch!(reply, self.add_nw_message(message)),
                WriteRequest::AddNwContact { contact, reply } => dispatch!(reply, self.add_nw_contact(contact)),
                WriteRequest::AddNwChat { chat, reply } => dispatch!(reply, self.add_nw_chat(chat)),
            }
        }

        log::info!("[DB-WRITER] loop exit");
    }
}