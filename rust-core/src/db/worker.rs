use std::path;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use rusqlite::{Connection, OpenFlags};
use anyhow::Result;

use crate::nw::NwDbRequest;
use crate::ui::UiDbRequest;

use super::{SCHEMA, DbWorker, WorkerImplemented};

#[derive(Debug)]
pub enum DbMode {
    ReadOnly,
    ReadWrite,
}

impl<TRequest: Send + 'static> DbWorker<TRequest> {

    fn start_conn(db_path: &path::Path, mode: DbMode) -> Result<Connection> {

        let flags = match mode {
            DbMode::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
            DbMode::ReadWrite => {
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX
            }
        };

        let conn = Connection::open_with_flags(db_path, flags)
            .map_err(|e| {
                log::error!("[DB] open failed: {}", e);
                e
            })?;

        if matches!(mode, DbMode::ReadWrite) {
            let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");
        }

        Ok(conn)
    }

    pub fn execute_schema(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA)
            .map_err(|e| {
                log::error!("[DB] schema failed: {}", e);
                e.into()
            })
    }
}

fn send_reply<T>(reply: oneshot::Sender<T>, result: T) {
    let _ = reply.send(result);
}

impl WorkerImplemented for DbWorker<UiDbRequest> {

    type Request = UiDbRequest;

    fn start(worker_rx: mpsc::Receiver<UiDbRequest>, db_path: PathBuf) -> Result<Self> {

        log::info!("[UI-WORKER] start db_path={:?}", db_path);

        let conn = Self::start_conn(&db_path, DbMode::ReadOnly)?;

        log::info!("[UI-WORKER] started");

        Ok(Self {
            worker_rx,
            conn
        })
    }

    fn request_loop(mut self) {

        log::info!("[UI-WORKER] loop start");

        while let Some(request) = self.worker_rx.blocking_recv() {
            match request {
                UiDbRequest::GetChats { reply } =>
                    send_reply(reply, self.get_chats()),

                UiDbRequest::GetChat { topic_id, reply } =>
                    send_reply(reply, self.get_chat(&topic_id)),

                UiDbRequest::GetChatMembers { topic_id, reply } =>
                    send_reply(reply, self.get_chat_members(&topic_id)),

                UiDbRequest::GetChatMessages { topic_id, reply } =>
                    send_reply(reply, self.get_chat_messages(&topic_id)),

                UiDbRequest::GetChatLastMessage { topic_id, reply } =>
                    send_reply(reply, self.get_chat_last_message(&topic_id)),

                UiDbRequest::GetChatWithMessages { topic_id, reply } =>
                    send_reply(reply, self.get_chat_with_messages(&topic_id)),

                UiDbRequest::GetContacts { reply } =>
                    send_reply(reply, self.get_contacts()),
            }
        }

        log::info!("[UI-WORKER] loop exit");
    }
}

impl WorkerImplemented for DbWorker<NwDbRequest> {

    type Request = NwDbRequest;

    fn start(worker_rx: mpsc::Receiver<NwDbRequest>, db_path: PathBuf) -> Result<Self> {

        log::info!("[NW-WORKER] start db_path={:?}", db_path);

        let conn = Self::start_conn(&db_path, DbMode::ReadWrite)?;

        let worker = Self {
            worker_rx,
            conn
        };

        worker.execute_schema()?;

        log::info!("[NW-WORKER] ready");

        Ok(worker)
    }

    fn request_loop(mut self) {

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