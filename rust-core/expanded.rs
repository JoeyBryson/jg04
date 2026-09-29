pub mod client {
    use tokio::sync::oneshot;
    use tokio::sync::mpsc;
    use crate::database::macros::db_requests;
    use crate::network::{NwChat, NwChatMember, NwContact, NwMessage, NwProfile};
    use crate::notifications::UiEvent::{
        ChatDataChanged, ChatHeadersChanged, ContactsChanged,
    };
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
    pub struct DbClient {
        reader_tx: mpsc::Sender<ReadRequest>,
        writer_tx: mpsc::Sender<WriteRequest>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DbClient {
        #[inline]
        fn clone(&self) -> DbClient {
            DbClient {
                reader_tx: ::core::clone::Clone::clone(&self.reader_tx),
                writer_tx: ::core::clone::Clone::clone(&self.writer_tx),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DbClient {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "DbClient",
                "reader_tx",
                &self.reader_tx,
                "writer_tx",
                &&self.writer_tx,
            )
        }
    }
    impl DbClient {
        pub(super) fn new(
            reader_tx: mpsc::Sender<ReadRequest>,
            writer_tx: mpsc::Sender<WriteRequest>,
        ) -> Self {
            DbClient { reader_tx, writer_tx }
        }
    }
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
    pub enum ReadRequest {
        GetUiChatHeaders {
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<UiChatHeader>>>,
        },
        GetUiChatHeader {
            topic_id: String,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<UiChatHeader>>,
        },
        GetUiChatMembers {
            topic_id: String,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
        },
        GetUiChatMessages {
            topic_id: String,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<UiMessage>>>,
        },
        GetUiChatLastMessage {
            topic_id: String,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Option<UiMessage>>>,
        },
        GetUiChatData {
            topic_id: String,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<UiChatData>>,
        },
        GetUiContacts {
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<UiContact>>>,
        },
        GetUiProfile {
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Option<UiProfile>>>,
        },
        GetNwProfile { reply: tokio::sync::oneshot::Sender<anyhow::Result<NwProfile>> },
        GetNwChats { reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<NwChat>>> },
        GetNwChatMembers {
            topic_id: TopicId,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<NwChatMember>>>,
        },
        GetNwChat {
            topic_id: TopicId,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<NwChat>>,
        },
        GetNwChatMessages {
            topic_id: TopicId,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<NwMessage>>>,
        },
        GetNwMessages {
            reply: tokio::sync::oneshot::Sender<anyhow::Result<Vec<NwMessage>>>,
        },
        GetNwContact {
            endpoint_id: EndpointId,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<NwContact>>,
        },
    }
    pub enum WriteRequest {
        SetNwProfile {
            profile: NwProfile,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
        },
        AddNwMessage {
            message: NwMessage,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
        },
        AddNwContact {
            contact: NwContact,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
        },
        AddNwChat {
            chat: NwChat,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
        },
        MarkNwChatMemberJoined {
            topic_id: TopicId,
            endpoint_id: EndpointId,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
        },
        AddUiContact {
            contact: UiContact,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
        },
        AddChatUi {
            contacts: Vec<UiContact>,
            name: Option<String>,
            reply: tokio::sync::oneshot::Sender<anyhow::Result<String>>,
        },
        ResetDatabase { reply: tokio::sync::oneshot::Sender<anyhow::Result<()>> },
    }
    impl crate::database::workers::DbReader {
        pub fn dispatch(&mut self, request: ReadRequest) {
            match request {
                ReadRequest::GetUiChatHeaders { reply } => {
                    let _ = reply.send(self.get_ui_chat_headers());
                }
                ReadRequest::GetUiChatHeader { topic_id, reply } => {
                    let _ = reply.send(self.get_ui_chat_header(&topic_id));
                }
                ReadRequest::GetUiChatMembers { topic_id, reply } => {
                    let _ = reply.send(self.get_ui_chat_members(&topic_id));
                }
                ReadRequest::GetUiChatMessages { topic_id, reply } => {
                    let _ = reply.send(self.get_ui_chat_messages(&topic_id));
                }
                ReadRequest::GetUiChatLastMessage { topic_id, reply } => {
                    let _ = reply.send(self.get_ui_last_chat_message(&topic_id));
                }
                ReadRequest::GetUiChatData { topic_id, reply } => {
                    let _ = reply.send(self.get_ui_chat_data(&topic_id));
                }
                ReadRequest::GetUiContacts { reply } => {
                    let _ = reply.send(self.get_ui_contacts());
                }
                ReadRequest::GetUiProfile { reply } => {
                    let _ = reply.send(self.get_ui_profile());
                }
                ReadRequest::GetNwProfile { reply } => {
                    let _ = reply.send(self.get_nw_profile());
                }
                ReadRequest::GetNwChats { reply } => {
                    let _ = reply.send(self.get_nw_chats());
                }
                ReadRequest::GetNwChatMembers { topic_id, reply } => {
                    let _ = reply.send(self.get_nw_chat_members(&topic_id));
                }
                ReadRequest::GetNwChat { topic_id, reply } => {
                    let _ = reply.send(self.get_nw_chat(&topic_id));
                }
                ReadRequest::GetNwChatMessages { topic_id, reply } => {
                    let _ = reply.send(self.get_nw_chat_messages(&topic_id));
                }
                ReadRequest::GetNwMessages { reply } => {
                    let _ = reply.send(self.get_nw_messages());
                }
                ReadRequest::GetNwContact { endpoint_id, reply } => {
                    let _ = reply.send(self.get_nw_contact(&endpoint_id));
                }
            }
        }
    }
    impl crate::database::workers::DbWriter {
        pub fn dispatch(&mut self, request: WriteRequest) {
            match request {
                WriteRequest::SetNwProfile { reply, profile } => {
                    let _ = reply.send(self.set_nw_profile(profile));
                }
                WriteRequest::AddNwMessage { reply, message } => {
                    let _ = reply.send(self.add_nw_message(message));
                }
                WriteRequest::AddNwContact { reply, contact } => {
                    let _ = reply.send(self.add_nw_contact(contact));
                }
                WriteRequest::AddNwChat { reply, chat } => {
                    let _ = reply.send(self.add_nw_chat(chat));
                }
                WriteRequest::MarkNwChatMemberJoined {
                    reply,
                    topic_id,
                    endpoint_id,
                } => {
                    let _ = reply
                        .send(self.mark_nw_chat_member_joined(topic_id, endpoint_id));
                }
                WriteRequest::AddUiContact { reply, contact } => {
                    let _ = reply.send(self.add_ui_contact(contact));
                }
                WriteRequest::AddChatUi { reply, contacts, name } => {
                    let _ = reply.send(self.add_chat_ui(contacts, name));
                }
                WriteRequest::ResetDatabase { reply } => {
                    let _ = reply.send(self.reset_database());
                }
            }
        }
    }
    impl DbClient {
        pub fn get_ui_chat_headers(
            &self,
        ) -> ::std::result::Result<Vec<UiChatHeader>, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiChatHeaders {
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_chat_headers(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        Vec<UiChatHeader>,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args.0.get_ui_chat_headers();
                        <::std::result::Result<
                            Vec<UiChatHeader>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            Vec<UiChatHeader>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADERS: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_chat_headers")
        .concat_bool(false)
        .concat_value(0u8)
        .concat(
            <::std::result::Result<
                Vec<UiChatHeader>,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADERS: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADERS
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADERS
        .into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_chat_headers() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADERS
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_chat_header(
            &self,
            topic_id: String,
        ) -> ::std::result::Result<UiChatHeader, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiChatHeader {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_chat_header(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        topic_id: <String as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        UiChatHeader,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <String as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(topic_id) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("topic_id", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args
                            .0
                            .get_ui_chat_header(uniffi_args.1);
                        <::std::result::Result<
                            UiChatHeader,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            UiChatHeader,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADER: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_chat_header")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("topic_id")
        .concat(<String as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                UiChatHeader,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADER: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADER
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADER
        .into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_chat_header() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_HEADER
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_chat_members(
            &self,
            topic_id: String,
        ) -> ::std::result::Result<Vec<UiContact>, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiChatMembers {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_chat_members(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        topic_id: <String as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        Vec<UiContact>,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <String as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(topic_id) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("topic_id", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args
                            .0
                            .get_ui_chat_members(uniffi_args.1);
                        <::std::result::Result<
                            Vec<UiContact>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            Vec<UiContact>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MEMBERS: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_chat_members")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("topic_id")
        .concat(<String as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                Vec<UiContact>,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MEMBERS: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MEMBERS
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MEMBERS
        .into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_chat_members() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MEMBERS
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_chat_messages(
            &self,
            topic_id: String,
        ) -> ::std::result::Result<Vec<UiMessage>, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiChatMessages {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_chat_messages(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        topic_id: <String as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        Vec<UiMessage>,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <String as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(topic_id) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("topic_id", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args
                            .0
                            .get_ui_chat_messages(uniffi_args.1);
                        <::std::result::Result<
                            Vec<UiMessage>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            Vec<UiMessage>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MESSAGES: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_chat_messages")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("topic_id")
        .concat(<String as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                Vec<UiMessage>,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MESSAGES: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MESSAGES
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MESSAGES
        .into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_chat_messages() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_MESSAGES
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_chat_last_message(
            &self,
            topic_id: String,
        ) -> ::std::result::Result<Option<UiMessage>, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiChatLastMessage {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_chat_last_message(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        topic_id: <String as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        Option<UiMessage>,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <String as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(topic_id) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("topic_id", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args
                            .0
                            .get_ui_chat_last_message(uniffi_args.1);
                        <::std::result::Result<
                            Option<UiMessage>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            Option<UiMessage>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_LAST_MESSAGE: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_chat_last_message")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("topic_id")
        .concat(<String as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                Option<UiMessage>,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_LAST_MESSAGE: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_LAST_MESSAGE
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_LAST_MESSAGE
        .into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_chat_last_message() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_LAST_MESSAGE
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_chat_data(
            &self,
            topic_id: String,
        ) -> ::std::result::Result<UiChatData, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiChatData {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_chat_data(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        topic_id: <String as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        UiChatData,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <String as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(topic_id) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("topic_id", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args
                            .0
                            .get_ui_chat_data(uniffi_args.1);
                        <::std::result::Result<
                            UiChatData,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            UiChatData,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_DATA: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_chat_data")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("topic_id")
        .concat(<String as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                UiChatData,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_DATA: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_DATA
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_DATA
        .into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_chat_data() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CHAT_DATA
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_contacts(
            &self,
        ) -> ::std::result::Result<Vec<UiContact>, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiContacts {
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_contacts(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        Vec<UiContact>,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args.0.get_ui_contacts();
                        <::std::result::Result<
                            Vec<UiContact>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            Vec<UiContact>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CONTACTS: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_contacts")
        .concat_bool(false)
        .concat_value(0u8)
        .concat(
            <::std::result::Result<
                Vec<UiContact>,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_CONTACTS: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CONTACTS
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CONTACTS.into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_contacts() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_CONTACTS
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_ui_profile(
            &self,
        ) -> ::std::result::Result<Option<UiProfile>, crate::ffi_error::FfiError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetUiProfile {
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_get_ui_profile(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        Option<UiProfile>,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args.0.get_ui_profile();
                        <::std::result::Result<
                            Option<UiProfile>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            Option<UiProfile>,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_PROFILE: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("get_ui_profile")
        .concat_bool(false)
        .concat_value(0u8)
        .concat(
            <::std::result::Result<
                Option<UiProfile>,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_GET_UI_PROFILE: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_PROFILE
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_PROFILE.into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_get_ui_profile() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_GET_UI_PROFILE
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn get_nw_profile(&self) -> ::std::result::Result<NwProfile, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetNwProfile {
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn get_nw_chats(
            &self,
        ) -> ::std::result::Result<Vec<NwChat>, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_async(
                    crate::database::client::ReadRequest::GetNwChats {
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            Ok(result)
        }
    }
    impl DbClient {
        pub fn get_nw_chats_sync(
            &self,
        ) -> ::std::result::Result<Vec<NwChat>, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_sync(
                    crate::database::client::ReadRequest::GetNwChats {
                        reply: tx,
                    },
                    rx,
                )?;
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn get_nw_chat_members(
            &self,
            topic_id: TopicId,
        ) -> ::std::result::Result<Vec<NwChatMember>, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_async(
                    crate::database::client::ReadRequest::GetNwChatMembers {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn get_nw_chat(
            &self,
            topic_id: TopicId,
        ) -> ::std::result::Result<NwChat, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_async(
                    crate::database::client::ReadRequest::GetNwChat {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn get_nw_chat_messages(
            &self,
            topic_id: TopicId,
        ) -> ::std::result::Result<Vec<NwMessage>, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_async(
                    crate::database::client::ReadRequest::GetNwChatMessages {
                        topic_id,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn get_nw_messages(
            &self,
        ) -> ::std::result::Result<Vec<NwMessage>, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_async(
                    crate::database::client::ReadRequest::GetNwMessages {
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn get_nw_contact(
            &self,
            endpoint_id: EndpointId,
        ) -> ::std::result::Result<NwContact, anyhow::Error> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_read_request_async(
                    crate::database::client::ReadRequest::GetNwContact {
                        endpoint_id,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            Ok(result)
        }
    }
    impl DbClient {
        pub fn set_profile(
            &self,
            profile: NwProfile,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_sync(
                    crate::database::client::WriteRequest::SetNwProfile {
                        profile,
                        reply: tx,
                    },
                    rx,
                )?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn add_nw_message(
            &self,
            message: NwMessage,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[
                ChatDataChanged {
                    topic_id: hex::encode(message.topic_id),
                },
                ChatHeadersChanged,
            ];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_async(
                    crate::database::client::WriteRequest::AddNwMessage {
                        message,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn add_nw_contact(
            &self,
            contact: NwContact,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[ContactsChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_async(
                    crate::database::client::WriteRequest::AddNwContact {
                        contact,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub fn add_nw_contact_sync(
            &self,
            contact: NwContact,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[ContactsChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_sync(
                    crate::database::client::WriteRequest::AddNwContact {
                        contact,
                        reply: tx,
                    },
                    rx,
                )?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn add_nw_chat(
            &self,
            chat: NwChat,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[ChatHeadersChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_async(
                    crate::database::client::WriteRequest::AddNwChat {
                        chat,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub fn add_nw_chat_sync(
            &self,
            chat: NwChat,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[ChatHeadersChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_sync(
                    crate::database::client::WriteRequest::AddNwChat {
                        chat,
                        reply: tx,
                    },
                    rx,
                )?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub async fn mark_nw_chat_member_joined(
            &self,
            topic_id: TopicId,
            endpoint_id: EndpointId,
        ) -> ::std::result::Result<(), anyhow::Error> {
            let __events: &[crate::notifications::UiEvent] = &[ChatHeadersChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_async(
                    crate::database::client::WriteRequest::MarkNwChatMemberJoined {
                        topic_id,
                        endpoint_id,
                        reply: tx,
                    },
                    rx,
                )
                .await?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    impl DbClient {
        pub fn add_ui_contact(
            &self,
            contact: UiContact,
        ) -> ::std::result::Result<(), crate::ffi_error::FfiError> {
            let __events: &[crate::notifications::UiEvent] = &[ContactsChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_sync(
                    crate::database::client::WriteRequest::AddUiContact {
                        contact,
                        reply: tx,
                    },
                    rx,
                )?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_add_ui_contact(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        contact: <UiContact as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        (),
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <UiContact as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(contact) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("contact", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args.0.add_ui_contact(uniffi_args.1);
                        <::std::result::Result<
                            (),
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            (),
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_UI_CONTACT: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("add_ui_contact")
        .concat_bool(false)
        .concat_value(1u8)
        .concat_str("contact")
        .concat(<UiContact as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                (),
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_ADD_UI_CONTACT: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_UI_CONTACT
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_UI_CONTACT.into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_add_ui_contact() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_UI_CONTACT
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn add_chat_ui(
            &self,
            contacts: Vec<UiContact>,
            name: Option<String>,
        ) -> ::std::result::Result<String, crate::ffi_error::FfiError> {
            let __events: &[crate::notifications::UiEvent] = &[ChatHeadersChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_sync(
                    crate::database::client::WriteRequest::AddChatUi {
                        contacts,
                        name,
                        reply: tx,
                    },
                    rx,
                )?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_add_chat_ui(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        contacts: <Vec<UiContact> as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        name: <Option<String> as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        String,
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
            match <Vec<
                UiContact,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(contacts) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("contacts", e));
                }
            },
            match <Option<String> as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(name) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("name", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args
                            .0
                            .add_chat_ui(uniffi_args.1, uniffi_args.2);
                        <::std::result::Result<
                            String,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            String,
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_CHAT_UI: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("add_chat_ui")
        .concat_bool(false)
        .concat_value(2u8)
        .concat_str("contacts")
        .concat(<Vec<UiContact> as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat_str("name")
        .concat(<Option<String> as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META)
        .concat_bool(false)
        .concat(
            <::std::result::Result<
                String,
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_ADD_CHAT_UI: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_CHAT_UI
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_CHAT_UI.into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_add_chat_ui() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_ADD_CHAT_UI
            .checksum();
        CHECKSUM
    }
    impl DbClient {
        pub fn reset_database(
            &self,
        ) -> ::std::result::Result<(), crate::ffi_error::FfiError> {
            let __events: &[crate::notifications::UiEvent] = &[ChatHeadersChanged];
            let (tx, rx) = tokio::sync::oneshot::channel();
            let result = self
                .send_write_request_sync(
                    crate::database::client::WriteRequest::ResetDatabase {
                        reply: tx,
                    },
                    rx,
                )?;
            for event in __events.iter().cloned() {
                crate::notifications::emit_ui_event(event);
            }
            Ok(result)
        }
    }
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_fn_method_dbclient_reset_database(
        uniffi_self_lowered: <::std::sync::Arc<
            DbClient,
        > as ::uniffi::Lift<crate::UniFfiTag>>::FfiType,
        call_status: &mut ::uniffi::RustCallStatus,
    ) -> <::std::result::Result<
        (),
        crate::ffi_error::FfiError,
    > as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
        let uniffi_lift_args = move || ::std::result::Result::Ok((
            match <::std::sync::Arc<
                DbClient,
            > as ::uniffi::Lift<crate::UniFfiTag>>::try_lift(uniffi_self_lowered) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => {
                    return ::std::result::Result::Err(("self", e));
                }
            },
        ));
        ::uniffi::rust_call(
            call_status,
            || {
                let result = match uniffi_lift_args() {
                    ::std::result::Result::Ok(uniffi_args) => {
                        let uniffi_result = uniffi_args.0.reset_database();
                        <::std::result::Result<
                            (),
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::lower_return(uniffi_result)
                    }
                    ::std::result::Result::Err((arg_name, error)) => {
                        <::std::result::Result<
                            (),
                            crate::ffi_error::FfiError,
                        > as ::uniffi::LowerReturn<
                            crate::UniFfiTag,
                        >>::handle_failed_lift(::uniffi::LiftArgsError {
                            arg_name,
                            error,
                        })
                    }
                };
                result
            },
        )
    }
    const UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_RESET_DATABASE: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(
            ::uniffi::metadata::codes::METHOD,
        )
        .concat_str("rust_api")
        .concat_str("DbClient")
        .concat_str("reset_database")
        .concat_bool(false)
        .concat_value(0u8)
        .concat(
            <::std::result::Result<
                (),
                crate::ffi_error::FfiError,
            > as ::uniffi::TypeId<crate::UniFfiTag>>::TYPE_ID_META,
        )
        .concat_long_str("");
    #[unsafe(no_mangle)]
    #[doc(hidden)]
    pub static UNIFFI_META_RUST_API_METHOD_DBCLIENT_RESET_DATABASE: [u8; UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_RESET_DATABASE
        .size] = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_RESET_DATABASE.into_array();
    #[doc(hidden)]
    #[unsafe(no_mangle)]
    pub extern "C" fn uniffi_rust_api_checksum_method_dbclient_reset_database() -> u16 {
        const CHECKSUM: u16 = UNIFFI_META_CONST_RUST_API_METHOD_DBCLIENT_RESET_DATABASE
            .checksum();
        CHECKSUM
    }
}
