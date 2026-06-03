use std::sync::{Arc, OnceLock};

static UI_CALLBACK: OnceLock<Arc<dyn UiEventListener>> =
    OnceLock::new();

#[derive(uniffi::Enum)]
pub enum UiEvent {
    // ChatListChanged,
    // ContactsChanged,
    ChatMessagesChanged {
        topic_id: Vec<u8>,
    },
}

#[uniffi::export(callback_interface)]
pub trait UiEventListener: Send + Sync {
    fn on_event(&self, event: UiEvent);
}

#[uniffi::export]
pub fn init_ui_events(
    callback: Box<dyn UiEventListener>
) {
    let arc: Arc<dyn UiEventListener> = Arc::from(callback);

    if UI_CALLBACK.set(arc).is_err() {
        log::warn!("ui callback already registered");
    }
}