use std::result::Result;
use tokio::sync::oneshot;

use tokio::sync::mpsc;
use super::requests::{ReadRequest, WriteRequest};

#[derive(uniffi::Object, Clone, Debug)]
pub struct DbClient {
    pub reader_tx: mpsc::Sender<ReadRequest>,
    pub writer_tx: mpsc::Sender<WriteRequest>
}

impl DbClient {
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
