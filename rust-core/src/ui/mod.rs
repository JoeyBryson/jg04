//! Types exposed to and consumed by the UI.
//!
//! These types define the data model returned across the FFI boundary. They
//! are derived from the application's internal and database representations,
//! with values converted into forms that are convenient for the UI to consume.
//!
//! Keeping these types separate from the internal network and database types
//! prevents UI concerns from leaking into the rest of the application and
//! gives the FFI boundary a stable interface.
//!
//! Database queries can construct these types directly, allowing data
//! transformation to happen within the Rust core before it crosses the FFI
//! boundary. This keeps the UI layer focused on presentation rather than
//! database-specific or domain-specific data conversion.

#[derive(uniffi::Enum, Debug)]
pub enum UiSender {
    Me,
    Other(UiContact),
}

#[derive(uniffi::Record, Debug)]
pub struct UiProfile {
    pub name: String,
    pub endpoint_id: String,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct UiContact {
    pub name: String,
    pub endpoint_id: String,
}


//members DOES include the self contact like NwChat
#[derive(uniffi::Record, Debug)]
pub struct UiChatHeader {
    pub name: Option<String>,
    pub members: Vec<UiContact>,
    pub topic_id: String,
    pub last_message: Option<UiMessage>,
}
#[derive(uniffi::Record, Debug)]
pub struct UiMessage {
    pub sender: UiSender,
    pub content: String,
    pub sent_at: i64,
}
#[derive(uniffi::Record, Debug)]
pub struct UiChatData {
    pub chat: UiChatHeader,
    pub messages: Vec<UiMessage>,
}
