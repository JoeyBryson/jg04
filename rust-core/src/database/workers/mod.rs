//! Database workers which run in their own threads and own their own database connections.
//!
//! The purpose of this design is to keep database connections alive somewhere that can be
//! accessed from anywhere in the app. Channels provide a natural way to queue database work
//! on a single connection.
//!
//! Using the Tokio variant [`tokio::sync::mpsc`] of the mpsc primitive allows for natural
//! integration for async database calls, allowing the app to do other work while the data is
//! being fetched or written. Sync database calls are also possible via the `.blocking_send`
//! and `.blocking_recv` methods.
//!
//! The [`DbReader`] and [`DbWriter`] workers each live on their own threads. This keeps writes
//! from blocking reads, as SQLite WAL mode allows multiple readers alongside one writer.
//! Writes can take orders of magnitude longer and UI responsiveness is a priority.
//!
//! This may be over-engineering, but it's done now.


use rusqlite::Connection;
use tokio::sync::mpsc;


mod nw_reads;
mod ui_reads;
mod writes;

#[cfg(test)]
mod nw_reads_test;
#[cfg(test)]
mod ui_reads_test;
#[cfg(test)]
mod writes_test;

use super::client::{ReadRequest, WriteRequest};

pub struct DbReader {
    rx: mpsc::Receiver<ReadRequest>,
    conn: Connection,
}

pub struct DbWriter {
    rx: mpsc::Receiver<WriteRequest>,
    conn: Connection,
}

impl DbReader {
    pub fn new(worker_rx: mpsc::Receiver<ReadRequest>, conn: Connection) -> Self {
        Self {
            rx: worker_rx,
            conn,
        }
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-READER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            // Dispatch is generated in super::client.rs using macros from super::macros.rs.
            // It is effectively just a match statement that matches the request enum to
            // internal methods.
            self.dispatch(request);
        }

        log::info!("[DB-READER] loop exit");
    }
}

impl DbWriter {
    pub fn new(worker_rx: mpsc::Receiver<WriteRequest>, conn: Connection) -> Self {
        Self {
            rx: worker_rx,
            conn,
        }
    }

    pub fn request_loop(mut self) {
        log::info!("[DB-WRITER] loop start");

        while let Some(request) = self.rx.blocking_recv() {
            self.dispatch(request);
        }

        log::info!("[DB-WRITER] loop exit");
    }
}

///Testing utilities
#[cfg(test)]
pub(super) mod test_utils {
    use iroh::{EndpointId, SecretKey};
    use iroh_gossip::proto::TopicId;
    use rusqlite::Connection;
    use tokio::sync::mpsc;
    use super::*;

    use crate::network::{
        NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile,
    };

    pub(super) fn connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();

        connection
            .execute_batch(include_str!("../sql/schema.sql"))
            .unwrap();

        connection
    }

    pub(super) fn new_writer() -> DbWriter {
        let (_, receiver) = mpsc::channel(1);

        DbWriter::new(receiver, connection())
    }

    pub(super) fn reader_from(connection: Connection) -> DbReader {
        let (_, receiver) = mpsc::channel(1);

        DbReader::new(receiver, connection)
    }

    pub(super) fn contact(seed: u8, name: &str) -> NwContact {
        NwContact {
            name: name.to_string(),
            endpoint_id: EndpointId::from(SecretKey::from_bytes(&[seed; 32]).public()),
        }
    }

    pub(super) fn profile() -> NwProfile {
        let secret_key = SecretKey::from_bytes(&[42; 32]);

        NwProfile {
            contact: NwContact {
                name: "Me".to_string(),
                endpoint_id: EndpointId::from(secret_key.public()),
            },
            secret_key,
        }
    }

    pub(super) fn chat(
        topic_seed: u8,
        members: Vec<NwChatMember>,
    ) -> NwChat {
        NwChat {
            name: Some("Test chat".to_string()),
            members,
            topic_id: TopicId::from_bytes([topic_seed; 32]),
        }
    }

    pub(super) fn member(
        contact: NwContact,
        status: NwChatMemberStatus,
    ) -> NwChatMember {
        NwChatMember { contact, status }
    }

    pub(super) fn message(
        topic_id: TopicId,
        endpoint_id: EndpointId,
        content: &str,
        sent_at: i64,
    ) -> NwMessage {
        NwMessage {
            topic_id,
            endpoint_id,
            content: content.to_string(),
            sent_at,
        }
    }

    pub(super) fn member_keys(
        chat: &NwChat,
    ) -> Vec<(Vec<u8>, NwChatMemberStatus)> {
        member_keys_from_members(&chat.members)
    }

    pub(super) fn member_keys_from_members(
        members: &[NwChatMember],
    ) -> Vec<(Vec<u8>, NwChatMemberStatus)> {
        let mut members = members
            .iter()
            .map(|member| {
                (
                    member.contact.endpoint_id.as_bytes().to_vec(),
                    member.status,
                )
            })
            .collect::<Vec<_>>();

        members.sort_by(|left, right| left.0.cmp(&right.0));

        members
    }
}