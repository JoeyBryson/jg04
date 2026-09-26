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

#[derive(uniffi::Record, Clone, Debug)]
pub struct UiContact {
    pub name: String,
    pub endpoint_id: String,
}

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
