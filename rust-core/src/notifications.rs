use std::sync::OnceLock;

#[derive(uniffi::Enum, Clone)]
pub enum UiEvent {
    ChatListChanged,
    ContactsChanged,
    ChatMessagesChanged { topic_id: String },
}

#[uniffi::export(callback_interface)]
pub trait UiEventListener: Send + Sync {
    fn on_event(&self, event: UiEvent);
}

static UI_EVENT_LISTENER: OnceLock<Box<dyn UiEventListener>> = OnceLock::new();

#[uniffi::export]
pub fn register_ui_event_listener(listener: Box<dyn UiEventListener>) -> bool {
    UI_EVENT_LISTENER.set(listener).is_ok()
}

pub fn emit_ui_event(event: UiEvent) {
    if let Some(listener) = UI_EVENT_LISTENER.get() {
        listener.on_event(event);
    } else {
        log::warn!("[EVENT-BUS] Drop event: No UI listener registered yet.");
    }
}