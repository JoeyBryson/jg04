//! This module provides UI reactivity to database insertions.
//!
//! UniFFI callback interfaces allow foreign-language code to implement a Rust
//! trait that Rust can call through the FFI boundary. Rust only knows that the
//! listener implements `on_event`; it does not need to know the concrete
//! implementation or what happens after the callback crosses the FFI boundary.
//!
//! `Box<dyn UiEventListener>` allows Rust to store the callback as a trait
//! object and invoke it through dynamic dispatch.
//!  
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


///holds the foreign implementation and makes it accessible from anywhere in the crate
static UI_EVENT_LISTENER: OnceLock<Box<dyn UiEventListener>> = OnceLock::new();

///sets the static variable UI_EVENT_LISTENER
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
