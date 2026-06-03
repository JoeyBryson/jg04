use std::path;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use rusqlite::{Connection, OpenFlags};
use anyhow::Result;

use crate::nw::NwDbRequest;
use crate::ui::UiDbRequest;

use super::{SCHEMA, DbWorker, WorkerImplemented};

pub enum DbMode {
    ReadOnly,
    ReadWrite,
}

impl<TRequest: Send + 'static> DbWorker<TRequest> {
    fn start_conn(db_path: &path::Path, mode: DbMode) -> Result<Connection> {
        let flags = match mode {
            DbMode::ReadOnly => {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            }
            DbMode::ReadWrite => {
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
            }
        };

        let conn = Connection::open_with_flags(db_path, flags)?;

        conn.execute_batch("PRAGMA synchronous = NORMAL;")?;

        if matches!(mode, DbMode::ReadWrite) {
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }

        Ok(conn)
    }

    pub fn execute_schema(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA)?;
        Ok(())
    }
}

fn send_reply<T>(reply: oneshot::Sender<T>, result: T) {
    if reply.send(result).is_err() {
        log::warn!("reply receiver dropped");
    }
}

impl WorkerImplemented for DbWorker<UiDbRequest> {
    type Request = UiDbRequest;

    fn start(
        worker_rx: mpsc::Receiver<UiDbRequest>,
        db_path: PathBuf,
    ) -> Result<Self> {
        let conn = Self::start_conn(&db_path, DbMode::ReadOnly)?;

        Ok(Self {
            worker_rx,
            conn,
            db_path,
        })
    }

    fn request_loop(mut self) {
        while let Some(request) = self.worker_rx.blocking_recv() {
            match request {
                UiDbRequest::GetChats { reply } => {
                    send_reply(reply, self.get_chats());
                }

                UiDbRequest::GetChat { topic_id, reply } => {
                    send_reply(reply, self.get_chat(&topic_id));
                }

                UiDbRequest::GetChatMembers { topic_id, reply } => {
                    send_reply(reply, self.get_chat_members(&topic_id));
                }

                UiDbRequest::GetChatMessages { topic_id, reply } => {
                    send_reply(reply, self.get_chat_messages(&topic_id));
                }

                UiDbRequest::GetChatLastMessage { topic_id, reply } => {
                    send_reply(reply, self.get_chat_last_message(&topic_id));
                }

                UiDbRequest::GetChatWithMessages { topic_id, reply } => {
                    send_reply(reply, self.get_chat_with_messages(&topic_id));
                }

                UiDbRequest::GetContacts { reply } => {
                    send_reply(reply, self.get_contacts());
                }
            }
        }
    }
}

impl WorkerImplemented for DbWorker<NwDbRequest> {
    type Request = NwDbRequest;

    fn start(
        worker_rx: mpsc::Receiver<NwDbRequest>,
        db_path: PathBuf,
    ) -> Result<Self> {
        // Delegate connection creation to the shared internal helper
        let conn = Self::start_conn(&db_path, DbMode::ReadWrite)?;

        let worker = Self {
            worker_rx,
            conn,
            db_path,
        };

        worker.execute_schema()?;

        Ok(worker)
    }

    fn request_loop(mut self) {
        while let Some(request) = self.worker_rx.blocking_recv() {
            match request {
                NwDbRequest::AddMessage { message, reply } => {
                    send_reply(reply, self.add_message(message));
                }

                NwDbRequest::AddContact { contact, reply } => {
                    send_reply(reply, self.add_contact(contact));
                }

                NwDbRequest::AddChat { chat, reply } => {
                    send_reply(reply, self.add_chat(chat));
                }

                NwDbRequest::GetChats { reply } => {
                    send_reply(reply, self.get_chats());
                }

                NwDbRequest::GetChatMembers { topic_id, reply } => {
                    send_reply(reply, self.get_chat_members(&topic_id));
                }

                NwDbRequest::GetChat { topic_id, reply } => {
                    send_reply(reply, self.get_chat(&topic_id));
                }

                NwDbRequest::GetChatMessages { topic_id, reply } => {
                    send_reply(reply, self.get_chat_messages(&topic_id));
                }

                NwDbRequest::GetMessages { reply } => {
                    send_reply(reply, self.get_messages());
                }
            }
        }
    }
}