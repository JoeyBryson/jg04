use tokio::sync::oneshot;

use super::requests::{ReadRequest, WriteRequest};
use tokio::sync::mpsc;

/// Public API for accessing the database.
///
/// Every application state read or write is exposed as a method of
/// `DbClient`, e.g.:
///
/// ```text
/// db_client.add_nw_chat(chat);
/// let chats: Vec<NwChat> = db_client.get_nw_chats();
/// ```
///
/// `DbClient` contains Tokio `mpsc` senders for the reader and writer
/// database workers. Requests are received by the corresponding worker,
/// processed, and the result is returned via a Tokio `oneshot` sender.
#[derive(uniffi::Object, Clone, Debug)]
pub struct DbClient {
    reader_tx: mpsc::Sender<ReadRequest>,
    writer_tx: mpsc::Sender<WriteRequest>,
}

impl DbClient {

    pub(super) fn new(
        reader_tx: mpsc::Sender<ReadRequest>,
        writer_tx: mpsc::Sender<WriteRequest>) -> Self {
        DbClient {
            reader_tx,
            writer_tx,
        }
    }

    pub(crate) fn send_read_request_sync<T>(
        &self,
        request: ReadRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.reader_tx.blocking_send(request)?;
        rx.blocking_recv()?
    }

    pub(crate) async fn send_read_request_async<T>(
        &self,
        request: ReadRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.reader_tx.send(request).await?;
        rx.await?
    }

    pub(crate) fn send_write_request_sync<T>(
        &self,
        request: WriteRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.writer_tx.blocking_send(request)?;
        rx.blocking_recv()?
    }

    pub(crate) async fn send_write_request_async<T>(
        &self,
        request: WriteRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.writer_tx.send(request).await?;
        rx.await?
    }
}
