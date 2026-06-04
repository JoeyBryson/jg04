
#[derive(uniffi::Enum, Clone)]
pub enum UiEvent {
    ChatListChanged,
    ContactsChanged,
    ChatMessagesChanged {
        topic_id: String,
    },
}

#[uniffi::export(callback_interface)]
pub trait UiEventListener: Send + Sync {
    fn on_event(&self, event: UiEvent);
}
