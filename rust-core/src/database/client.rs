use tokio::sync::oneshot;
use tokio::sync::mpsc;
use crate::database::macros::db_requests;
use crate::network::{NwChat, NwChatMember, NwContact, NwMessage, NwProfile};
use crate::notifications::UiEvent::{ChatDataChanged, ChatHeadersChanged, ContactsChanged};
use crate::ui::{UiChatData, UiChatHeader, UiContact, UiMessage, UiProfile};
use iroh::EndpointId;
use iroh_gossip::TopicId;

/// Public API for accessing the database.
///
/// Every application state read or write is exposed as a method of
/// `DbClient`, e.g.:
///
/// ```text
/// db_client.add_nw_chat(chat);
/// let chats: Vec<NwChat> = db_client.get_nw_chats();
/// ```
///
/// `DbClient` contains Tokio `mpsc` senders for the reader and writer
/// database workers. Requests are received by the corresponding worker,
/// processed, and the result is returned via a Tokio `oneshot` sender.
#[derive(uniffi::Object, Clone, Debug)]
pub struct DbClient {
    reader_tx: mpsc::Sender<ReadRequest>,
    writer_tx: mpsc::Sender<WriteRequest>,
}

impl DbClient {

    pub(super) fn new(
        reader_tx: mpsc::Sender<ReadRequest>,
        writer_tx: mpsc::Sender<WriteRequest>) -> Self {
        DbClient {
            reader_tx,
            writer_tx,
        }
    }
}


//request sending and response receiving helper functions.
//Sync and Async variants are available to allow us the flexibility to use DbClient in 
// different contexts while only creating the methods that are actually needed

impl DbClient {
        fn send_read_request_sync<T>(
        &self,
        request: ReadRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.reader_tx.blocking_send(request)?;
        rx.blocking_recv()?
    }

    async fn send_read_request_async<T>(
        &self,
        request: ReadRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.reader_tx.send(request).await?;
        rx.await?
    }

    fn send_write_request_sync<T>(
        &self,
        request: WriteRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.writer_tx.blocking_send(request)?;
        rx.blocking_recv()?
    }

    async fn send_write_request_async<T>(
        &self,
        request: WriteRequest,
        rx: oneshot::Receiver<anyhow::Result<T>>,
    ) -> anyhow::Result<T> {
        self.writer_tx.send(request).await?;
        rx.await?
    }
}



// Add a DB call: write the query fn in reads.rs/writes.rs, then add one line
// below. Modes: ffi_sync/ffi_async (uniffi-exported) or sync/async (internal
// Rust-only); writes also list `emits [...]` UiEvents to fire on success.
// Remebember all arguments and return types for ffi methods must be uniffi exported types
//
// Line shape:
//   Variant(field: Type, ...) -> ReturnType => query_fn { mode client_method, ... } [emits [...]];
//     Variant       - enum variant name (ReadRequest::Variant / WriteRequest::Variant),
//     - this should be a camel case match of the function name
//     field: Type   - request arguments, forwarded as-is to query_fn
//     ReturnType    - what query_fn returns (wrapped in Result by the macro)
//     query_fn      - the fn in reads.rs/writes.rs that does the actual DB work
//     mode          - ffi_sync/ffi_async/sync/async (see above)
//     client_method - name of the generated DbClient method for that mode
//     emits [...]   - (writes only) UiEvents fired via emit_ui_event on success

//Note: the ffi_async variant is also available, Uniffi uses foreign language native aync runtimes
// so these would not mesh at all with our internal tokio runtime. The option is included simply for 
// the sake of symmetry and curiosity and is completely untested. But it's usefulness can't be completely 
// ruled out yet  

db_requests! {
    reads {
        GetUiChatHeaders() -> Vec<UiChatHeader> => get_ui_chat_headers { ffi_sync get_ui_chat_headers };
        GetUiChatHeader(topic_id: String) -> UiChatHeader => get_ui_chat_header { ffi_sync get_ui_chat_header };
        GetUiChatMembers(topic_id: String) -> Vec<UiContact> => get_ui_chat_members { ffi_sync get_ui_chat_members };
        GetUiChatMessages(topic_id: String) -> Vec<UiMessage> => get_ui_chat_messages { ffi_sync get_ui_chat_messages };
        GetUiChatLastMessage(topic_id: String) -> Option<UiMessage> => get_ui_last_chat_message { ffi_sync get_ui_chat_last_message };
        GetUiChatData(topic_id: String) -> UiChatData => get_ui_chat_data { ffi_sync get_ui_chat_data };
        GetUiContacts() -> Vec<UiContact> => get_ui_contacts { ffi_sync get_ui_contacts };
        GetUiProfile() -> Option<UiProfile> => get_ui_profile { ffi_sync get_ui_profile };

        GetNwProfile() -> NwProfile => get_nw_profile { sync get_nw_profile };
        GetNwChats() -> Vec<NwChat> => get_nw_chats { async get_nw_chats, sync get_nw_chats_sync };
        GetNwChatMembers(topic_id: TopicId) -> Vec<NwChatMember> => get_nw_chat_members { async get_nw_chat_members };
        GetNwChat(topic_id: TopicId) -> NwChat => get_nw_chat { async get_nw_chat };
        GetNwChatMessages(topic_id: TopicId) -> Vec<NwMessage> => get_nw_chat_messages { async get_nw_chat_messages };
        GetNwMessages() -> Vec<NwMessage> => get_nw_messages { async get_nw_messages };
        GetNwContact(endpoint_id: EndpointId) -> NwContact => get_nw_contact {async get_nw_contact};
    }
    writes {
        SetNwProfile(profile: NwProfile) -> () => set_nw_profile { sync set_profile } emits [];
        AddNwMessage(message: NwMessage) -> () => add_nw_message { async add_nw_message }
            emits [ChatDataChanged { topic_id: hex::encode(message.topic_id) }, ChatHeadersChanged];
        AddNwContact(contact: NwContact) -> () => add_nw_contact { async add_nw_contact, sync add_nw_contact_sync }
            emits [ContactsChanged];
        AddNwChat(chat: NwChat) -> () => add_nw_chat { async add_nw_chat, sync add_nw_chat_sync }
            emits [ChatHeadersChanged];
        MarkNwChatMemberJoined(topic_id: TopicId, endpoint_id: EndpointId) -> () => mark_nw_chat_member_joined { async mark_nw_chat_member_joined }
            emits [ChatHeadersChanged];
        AddUiContact(contact: UiContact) -> () => add_ui_contact { ffi_sync add_ui_contact }
            emits [ContactsChanged];
        AddChatUi(contacts: Vec<UiContact>, name: Option<String>) -> String => add_chat_ui { ffi_sync add_chat_ui }
            emits [ChatHeadersChanged];
        ResetDatabase() -> () => reset_database {ffi_sync reset_database}
            emits [ChatHeadersChanged];
    }
}
