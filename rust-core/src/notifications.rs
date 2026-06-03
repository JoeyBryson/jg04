use tokio::sync::mpsc;
use crate::ui::UiDbRequest;

use std::sync::Arc;

#[derive(uniffi::Enum)]
pub enum UiEvent {
    ChatListChanged,
    ContactsChanged,
    ChatMessagesChanged {
        topic_id: Vec<u8>,
    },
}

#[uniffi::export(callback_interface)]
pub trait UiEventListener: Send + Sync {
    fn on_event(&self, event: UiEvent);
}

#[derive(uniffi::Object)]
pub struct UiDbClient {
    worker_tx: mpsc::Sender<UiDbRequest>,
    ui_listener: Arc<dyn UiEventListener>
}


#[uniffi::export]
pub fn init_ui_events(
    callback: Box<dyn UiEventListener>
) {
    let arc: Arc<dyn UiEventListener> = Arc::from(callback);
}