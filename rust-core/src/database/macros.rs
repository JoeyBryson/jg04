//! Macros for generating DbClient methods, DbRequest enum, and DbWorker match statements for calling internal methods
//!
//! DbClient.method <-> mpsc send request<DbRequest> -> DbWorker mpsc receive request <-> match statement over DbRequest <-> dbWorker.method <-> Database
//!                     receive oneshot response     <- send Oneshot response
//!
//! I would not mess with this file, it's purpose is to reduce boilerplate when adding new db calls.
//!


// Generates a single DbClient read method for a specific execution mode.
//
// For example:
//
// db_read_method_body!(
//     {},
//     {},
//     send_read_request_sync,
//     {},
//     anyhow::Error,
//     GetNwChat,
//     get_nw_chat,
//     NwChat,
//     (topic_id: TopicId)
// );
//
// expands roughly to:
//
// impl DbClient {
//     pub fn get_nw_chat(
//         &self,
//         topic_id: TopicId,
//     ) -> Result<NwChat, anyhow::Error> {
//         let (tx, rx) = tokio::sync::oneshot::channel();
//         let result = self.send_read_request_sync(
//             ReadRequest::GetNwChat {
//                 topic_id,
//                 reply: tx,
//             },
//             rx,
//         )?;
//         Ok(result)
//     }
// }
macro_rules! db_read_method_body {
    (
        {$($attributes:tt)*},
        {$($asyncness:tt)*},
        $send_method:ident,
        {$($await:tt)*},
        $error_type:ty,
        $request_variant:ident,
        $client_method:ident,
        $return_type:ty,
        ($($field_name:ident : $field_type:ty),*)
    ) => {
        $($attributes)*
        impl DbClient {
            pub $($asyncness)* fn $client_method(
                &self,
                $($field_name: $field_type),*
            ) -> ::std::result::Result<$return_type, $error_type> {
                let (tx, rx) = tokio::sync::oneshot::channel();

                let result = self.$send_method(
                    crate::database::client::ReadRequest::$request_variant {
                        $($field_name,)*
                        reply: tx,
                    },
                    rx,
                ) $($await)* ?;

                Ok(result)
            }
        }
    };
}


// Selects how a read method is exposed and executed.
//
// Each mode delegates to db_read_method_body! with different
// visibility, execution, and error-handling settings.
//
// For example:
//
// db_read_mode!(
//     async,
//     GetNwChat,
//     get_nw_chat,
//     NwChat,
//     (topic_id: TopicId)
// );
//
// expands roughly to:
//
// impl DbClient {
//     pub async fn get_nw_chat(
//         &self,
//         topic_id: TopicId,
//     ) -> Result<NwChat, anyhow::Error> {
//         let (tx, rx) = tokio::sync::oneshot::channel();
//         let result = self.send_read_request_async(
//             ReadRequest::GetNwChat {
//                 topic_id,
//                 reply: tx,
//             },
//             rx,
//         ).await?;
//         Ok(result)
//     }
// }
macro_rules! db_read_mode {
    (ffi_sync, $($arguments:tt)*) => {
        crate::database::macros::db_read_method_body!(
            {#[uniffi::export]},
            {},
            send_read_request_sync,
            {},
            crate::ffi_error::FfiError,
            $($arguments)*
        );
    };

    (ffi_async, $($arguments:tt)*) => {
        crate::database::macros::db_read_method_body!(
            {#[uniffi::export]},
            {async},
            send_read_request_async,
            {.await},
            crate::ffi_error::FfiError,
            $($arguments)*
        );
    };

    (sync, $($arguments:tt)*) => {
        crate::database::macros::db_read_method_body!(
            {},
            {},
            send_read_request_sync,
            {},
            anyhow::Error,
            $($arguments)*
        );
    };

    (async, $($arguments:tt)*) => {
        crate::database::macros::db_read_method_body!(
            {},
            {async},
            send_read_request_async,
            {.await},
            anyhow::Error,
            $($arguments)*
        );
    };
}


// Generates a single DbClient write method, including its UI events.
//
// For example:
//
// db_write_method_body!(
//     {},
//     {async},
//     send_write_request_async,
//     {.await},
//     anyhow::Error,
//     AddNwMessage,
//     add_nw_message,
//     (),
//     (message: NwMessage),
//     [ChatDataChanged]
// );
//
// expands roughly to:
//
// impl DbClient {
//     pub async fn add_nw_message(
//         &self,
//         message: NwMessage,
//     ) -> Result<(), anyhow::Error> {
//         let __events: &[UiEvent] = &[ChatDataChanged];
//         let (tx, rx) = tokio::sync::oneshot::channel();
//
//         let result = self.send_write_request_async(
//             WriteRequest::AddNwMessage {
//                 message,
//                 reply: tx,
//             },
//             rx,
//         ).await?;
//
//         for event in __events.iter().cloned() {
//             emit_ui_event(event);
//         }
//
//         Ok(result)
//     }
// }
macro_rules! db_write_method_body {
    (
        {$($attributes:tt)*},
        {$($asyncness:tt)*},
        $send_method:ident,
        {$($await:tt)*},
        $error_type:ty,
        $request_variant:ident,
        $client_method:ident,
        $return_type:ty,
        ($($field_name:ident : $field_type:ty),*),
        [$($event:expr),*]
    ) => {
        $($attributes)*
        impl DbClient {
            pub $($asyncness)* fn $client_method(
                &self,
                $($field_name: $field_type),*
            ) -> ::std::result::Result<$return_type, $error_type> {
                let __events: &[crate::notifications::UiEvent] = &[$($event),*];
                let (tx, rx) = tokio::sync::oneshot::channel();

                let result = self.$send_method(
                    crate::database::client::WriteRequest::$request_variant {
                        $($field_name,)*
                        reply: tx,
                    },
                    rx,
                ) $($await)* ?;

                for event in __events.iter().cloned() {
                    crate::notifications::emit_ui_event(event);
                }

                Ok(result)
            }
        }
    };
}


// Selects how a write method is exposed and executed.
//
// For example:
//
// db_write_mode!(
//     async,
//     AddNwMessage,
//     add_nw_message,
//     (),
//     (message: NwMessage),
//     [ChatDataChanged]
// );
//
// expands roughly to:
//
// impl DbClient {
//     pub async fn add_nw_message(
//         &self,
//         message: NwMessage,
//     ) -> Result<(), anyhow::Error> {
//         let __events: &[UiEvent] = &[ChatDataChanged];
//         let (tx, rx) = tokio::sync::oneshot::channel();
//
//         let result = self.send_write_request_async(
//             WriteRequest::AddNwMessage {
//                 message,
//                 reply: tx,
//             },
//             rx,
//         ).await?;
//
//         for event in __events.iter().cloned() {
//             emit_ui_event(event);
//         }
//
//         Ok(result)
//     }
// }
macro_rules! db_write_mode {
    (ffi_sync, $($arguments:tt)*) => {
        crate::database::macros::db_write_method_body!(
            {#[uniffi::export]},
            {},
            send_write_request_sync,
            {},
            crate::ffi_error::FfiError,
            $($arguments)*
        );
    };

    (ffi_async, $($arguments:tt)*) => {
        crate::database::macros::db_write_method_body!(
            {#[uniffi::export]},
            {async},
            send_write_request_async,
            {.await},
            crate::ffi_error::FfiError,
            $($arguments)*
        );
    };

    (sync, $($arguments:tt)*) => {
        crate::database::macros::db_write_method_body!(
            {},
            {},
            send_write_request_sync,
            {},
            anyhow::Error,
            $($arguments)*
        );
    };

    (async, $($arguments:tt)*) => {
        crate::database::macros::db_write_method_body!(
            {},
            {async},
            send_write_request_async,
            {.await},
            anyhow::Error,
            $($arguments)*
        );
    };
}


// Recursively generates all requested read methods for a database operation.
//
// For example:
//
// db_read_modes!(
//     GetNwChats,
//     Vec<NwChat>,
//     (),
//     async get_nw_chats,
//     sync get_nw_chats_sync
// );
//
// expands to two calls to db_read_mode!:
//
// db_read_mode!(
//     async,
//     GetNwChats,
//     get_nw_chats,
//     Vec<NwChat>,
//     ()
// );
//
// db_read_mode!(
//     sync,
//     GetNwChats,
//     get_nw_chats_sync,
//     Vec<NwChat>,
//     ()
// );
macro_rules! db_read_modes {
    (
        $request_variant:ident,
        $return_type:ty,
        $fields:tt,
        $mode:ident $client_method:ident
        $(, $($remaining_modes:tt)*)?
    ) => {
        crate::database::macros::db_read_mode!(
            $mode,
            $request_variant,
            $client_method,
            $return_type,
            $fields
        );

        $(
            crate::database::macros::db_read_modes!(
                $request_variant,
                $return_type,
                $fields,
                $($remaining_modes)*
            );
        )?
    };
}


// Recursively generates all requested write methods for a database operation.
//
// For example:
//
// db_write_modes!(
//     AddNwContact,
//     (),
//     (contact: NwContact),
//     [ContactsChanged],
//     async add_nw_contact,
//     sync add_nw_contact_sync
// );
//
// expands to two calls to db_write_mode!:
//
// db_write_mode!(
//     async,
//     AddNwContact,
//     add_nw_contact,
//     (),
//     (contact: NwContact),
//     [ContactsChanged]
// );
//
// db_write_mode!(
//     sync,
//     AddNwContact,
//     add_nw_contact_sync,
//     (),
//     (contact: NwContact),
//     [ContactsChanged]
// );
macro_rules! db_write_modes {
    (
        $request_variant:ident,
        $return_type:ty,
        $fields:tt,
        $events:tt,
        $mode:ident $client_method:ident
        $(, $($remaining_modes:tt)*)?
    ) => {
        crate::database::macros::db_write_mode!(
            $mode,
            $request_variant,
            $client_method,
            $return_type,
            $fields,
            $events
        );

        $(
            crate::database::macros::db_write_modes!(
                $request_variant,
                $return_type,
                $fields,
                $events,
                $($remaining_modes)*
            );
        )?
    };
}


// Defines database requests, worker dispatch, and the corresponding DbClient API
// from a single list of database operations.
//
// For example:
//
// db_requests! {
//     reads {
//         GetNwChat(topic_id: TopicId) -> NwChat => get_nw_chat {
//             async get_nw_chat
//         };
//     }
//     writes {
//         AddNwMessage(message: NwMessage) -> () => add_nw_message {
//             async add_nw_message
//         } emits [ChatDataChanged];
//     }
// }
//
// generates the following pieces:
//
// pub enum ReadRequest {
//     GetNwChat {
//         topic_id: TopicId,
//         reply: oneshot::Sender<anyhow::Result<NwChat>>,
//     },
// }
//
// pub enum WriteRequest {
//     AddNwMessage {
//         message: NwMessage,
//         reply: oneshot::Sender<anyhow::Result<()>>,
//     },
// }
//
// impl DbReader {
//     pub fn dispatch(&mut self, request: ReadRequest) {
//         match request {
//             ReadRequest::GetNwChat { topic_id, reply } => {
//                 let _ = reply.send(self.get_nw_chat(&topic_id));
//             }
//         }
//     }
// }
//
// impl DbWriter {
//     pub fn dispatch(&mut self, request: WriteRequest) {
//         match request {
//             WriteRequest::AddNwMessage { reply, message } => {
//                 let _ = reply.send(self.add_nw_message(message));
//             }
//         }
//     }
// }
//
// impl DbClient {
//     pub async fn get_nw_chat(
//         &self,
//         topic_id: TopicId,
//     ) -> Result<NwChat, anyhow::Error> {
//         let (tx, rx) = tokio::sync::oneshot::channel();
//
//         let result = self.send_read_request_async(
//             ReadRequest::GetNwChat {
//                 topic_id,
//                 reply: tx,
//             },
//             rx,
//         ).await?;
//
//         Ok(result)
//     }
//
//     pub async fn add_nw_message(
//         &self,
//         message: NwMessage,
//     ) -> Result<(), anyhow::Error> {
//         let __events: &[UiEvent] = &[ChatDataChanged];
//         let (tx, rx) = tokio::sync::oneshot::channel();
//
//         let result = self.send_write_request_async(
//             WriteRequest::AddNwMessage {
//                 message,
//                 reply: tx,
//             },
//             rx,
//         ).await?;
//
//         for event in __events.iter().cloned() {
//             emit_ui_event(event);
//         }
//
//         Ok(result)
//     }
// }
macro_rules! db_requests {
    (
        reads {
            $(
                $read_request_variant:ident (
                    $($read_field_name:ident : $read_field_type:ty),* $(,)?
                )
                -> $read_return_type:ty
                => $read_worker_method:ident {
                    $($read_mode:ident $read_client_method:ident),* $(,)?
                };
            )*
        }

        writes {
            $(
                $write_request_variant:ident (
                    $($write_field_name:ident : $write_field_type:ty),* $(,)?
                )
                -> $write_return_type:ty
                => $write_worker_method:ident {
                    $($write_mode:ident $write_client_method:ident),* $(,)?
                }
                emits [$($write_event:expr),* $(,)?];
            )*
        }
    ) => {
        // Carry database requests from DbClient to the appropriate worker.
        pub enum ReadRequest {
            $(
                $read_request_variant {
                    $($read_field_name: $read_field_type,)*
                    reply: tokio::sync::oneshot::Sender<
                        anyhow::Result<$read_return_type>
                    >,
                },
            )*
        }

        pub enum WriteRequest {
            $(
                $write_request_variant {
                    $($write_field_name: $write_field_type,)*
                    reply: tokio::sync::oneshot::Sender<
                        anyhow::Result<$write_return_type>
                    >,
                },
            )*
        }

        // Dispatch each request to the corresponding database operation.
        impl crate::database::workers::DbReader {
            pub fn dispatch(&mut self, request: ReadRequest) {
                match request {
                    $(
                        ReadRequest::$read_request_variant {
                            $($read_field_name,)*
                            reply
                        } => {
                            let _ = reply.send(
                                self.$read_worker_method(
                                    $(&$read_field_name),*
                                )
                            );
                        }
                    )*
                }
            }
        }

        impl crate::database::workers::DbWriter {
            pub fn dispatch(&mut self, request: WriteRequest) {
                match request {
                    $(
                        WriteRequest::$write_request_variant {
                            reply,
                            $($write_field_name,)*
                        } => {
                            let _ = reply.send(
                                self.$write_worker_method(
                                    $($write_field_name),*
                                )
                            );
                        }
                    )*
                }
            }
        }

        // Generate the requested DbClient methods for each execution mode.
        $(
            crate::database::macros::db_read_modes!(
                $read_request_variant,
                $read_return_type,
                ($($read_field_name: $read_field_type),*),
                $($read_mode $read_client_method),+
            );
        )*

        $(
            crate::database::macros::db_write_modes!(
                $write_request_variant,
                $write_return_type,
                ($($write_field_name: $write_field_type),*),
                [$($write_event),*],
                $($write_mode $write_client_method),+
            );
        )*
    };
}

pub(crate) use db_read_method_body;
pub(crate) use db_read_mode;
pub(crate) use db_read_modes;
pub(crate) use db_requests;
pub(crate) use db_write_method_body;
pub(crate) use db_write_mode;
pub(crate) use db_write_modes;