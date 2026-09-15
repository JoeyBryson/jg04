use crate::network::{NwChat, NwContact, NwMessage, NwProfile};
use crate::ui::{UiMessage, UiContact, UiChatHeader, UiChatData, UiProfile};
use crate::database::macros::db_requests;
use crate::notifications::UiEvent::{ChatHeadersChanged, ChatDataChanged, ContactsChanged};
use iroh::EndpointId;
//don't delete, needed
use crate::database::client::DbClient;

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

db_requests! {
    reads {
        // ProfileExists() -> bool => profile_exists { ffi_sync profile_exists };
        GetUiChatHeaders() -> Vec<UiChatHeader> => get_ui_chat_headers { ffi_sync get_ui_chat_headers };
        GetUiChatHeader(topic_id: String) -> UiChatHeader => get_ui_chat_header { ffi_sync get_ui_chat_header };
        GetUiChatMembers(topic_id: String) -> Vec<UiContact> => get_ui_chat_members { ffi_sync get_ui_chat_members };
        GetUiChatMessages(topic_id: String) -> Vec<UiMessage> => get_ui_chat_messages { ffi_sync get_ui_chat_messages };
        GetUiChatLastMessage(topic_id: String) -> Option<UiMessage> => get_ui_last_chat_message { ffi_sync get_ui_chat_last_message };
        GetUiChatData(topic_id: String) -> UiChatData => get_ui_chat_data { ffi_sync get_ui_chat_data };
        GetUiContacts() -> Vec<UiContact> => get_ui_contacts { ffi_sync get_ui_contacts };
        GetUiProfile() -> UiProfile => get_ui_profile { ffi_sync get_ui_profile };

        GetNwProfile() -> NwProfile => get_nw_profile { sync get_nw_profile };
        GetNwChats() -> Vec<NwChat> => get_nw_chats { async get_nw_chats };
        GetNwChatMembers(topic_id: Vec<u8>) -> Vec<NwContact> => get_nw_chat_members { async get_nw_chat_members };
        GetNwChat(topic_id: Vec<u8>) -> NwChat => get_nw_chat { async get_nw_chat };
        GetNwChatMessages(topic_id: Vec<u8>) -> Vec<NwMessage> => get_nw_chat_messages { async get_nw_chat_messages };
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
        AddUiContact(contact: UiContact) -> () => add_ui_contact { ffi_sync add_ui_contact } 
            emits [ContactsChanged];
        AddChatUi(contacts: Vec<UiContact>, name: Option<String>) -> String => add_chat_ui { ffi_sync add_chat_ui }
            emits [ChatHeadersChanged];
        ResetDatabase() -> () => reset_database {ffi_sync reset_database} 
            emits [ChatHeadersChanged];
    }
}



