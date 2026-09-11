
#[derive(uniffi::Enum)]
pub enum UiSender {
    Me,
    Other(UiContact)
}

#[derive(uniffi::Record)]
pub struct UiProfile {
    pub endpoint_id: String,
}

#[derive(uniffi::Record)]
pub struct UiContact {
    pub name: String,
    pub endpoint_id: String,
}

#[derive(uniffi::Record)]
pub struct UiChatHeader {
    pub name: Option<String>,
    pub members: Vec<UiContact>,
    pub topic_id: String,
    pub last_message: Option<UiMessage>
}
#[derive(uniffi::Record)]
pub struct UiMessage {
    pub sender: UiSender,
    pub content: String,
    pub sent_at: i64,
}
#[derive(uniffi::Record)]
pub struct UiChatData {
    pub chat: UiChatHeader,
    pub messages: Vec<UiMessage>
}


