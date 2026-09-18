use crate::ffi_error::FfiError;
use std::sync::OnceLock;

#[derive(uniffi::Enum, Clone)]
pub enum UiEvent {
    ChatHeadersChanged,
    ContactsChanged,
    ChatDataChanged { topic_id: String },
}

#[uniffi::export(callback_interface)]
pub trait UiEventListener: Send + Sync {
    fn on_event(&self, event: UiEvent);
}

static UI_EVENT_LISTENER: OnceLock<Box<dyn UiEventListener>> = OnceLock::new();

#[uniffi::export]
pub fn register_ui_event_listener(listener: Box<dyn UiEventListener>) -> Result<(), FfiError> {
    UI_EVENT_LISTENER
        .set(listener)
        .map_err(|_| FfiError::Internal {
            msg: "UI event listener already registered".to_string(),
        })
}

pub fn emit_ui_event(event: UiEvent) {
    if let Some(listener) = UI_EVENT_LISTENER.get() {
        listener.on_event(event);
    } else {
        log::warn!("[EVENT-BUS] Drop event: No UI listener registered yet.");
    }
}
