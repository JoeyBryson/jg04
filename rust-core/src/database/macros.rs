
macro_rules! db_read_method_body {
    ({$($attr:tt)*}, {$($asyncness:tt)*}, $send_fn:ident, {$($await_kw:tt)*}, $err_ty:ty, $variant:ident, $client_fn:ident, $ret:ty, ($($field:ident : $ty:ty),*)) => {
        $($attr)*
        impl DbClient {
            pub $($asyncness)* fn $client_fn(&self, $($field: $ty),*) -> ::std::result::Result<$ret, $err_ty> {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let result = self.$send_fn(crate::database::requests::ReadRequest::$variant { $($field,)* reply: tx }, rx) $($await_kw)* ?;
                Ok(result)
            }
        }
    };
}


macro_rules! db_read_mode {
    (ffi_sync, $($rest:tt)*) => {
        crate::database::macros::db_read_method_body!({#[uniffi::export]}, {}, send_read_request_sync, {}, crate::ffi_error::FfiError, $($rest)*);
    };
    (ffi_async, $($rest:tt)*) => {
        crate::database::macros::db_read_method_body!({#[uniffi::export]}, {async}, send_read_request_async, {.await}, crate::ffi_error::FfiError, $($rest)*);
    };
    (sync, $($rest:tt)*) => {
        crate::database::macros::db_read_method_body!({}, {}, send_read_request_sync, {}, anyhow::Error, $($rest)*);
    };
    (async, $($rest:tt)*) => {
        crate::database::macros::db_read_method_body!({}, {async}, send_read_request_async, {.await}, anyhow::Error, $($rest)*);
    };
}


macro_rules! db_write_method_body {
    ({$($attr:tt)*}, {$($asyncness:tt)*}, $send_fn:ident, {$($await_kw:tt)*}, $err_ty:ty, $variant:ident, $client_fn:ident, $ret:ty, ($($field:ident : $ty:ty),*), [$($event:expr),*]) => {
        $($attr)*
        impl DbClient {
            pub $($asyncness)* fn $client_fn(&self, $($field: $ty),*) -> ::std::result::Result<$ret, $err_ty> {
                let __events: &[crate::notifications::UiEvent] = &[$($event),*];
                let (tx, rx) = tokio::sync::oneshot::channel();
                let result = self.$send_fn(crate::database::requests::WriteRequest::$variant { $($field,)* reply: tx }, rx) $($await_kw)* ?;
                for event in __events.iter().cloned() {
                    crate::notifications::emit_ui_event(event);
                }
                Ok(result)
            }
        }
    };
}

macro_rules! db_write_mode {
    (ffi_sync, $($rest:tt)*) => {
        crate::database::macros::db_write_method_body!({#[uniffi::export]}, {}, send_write_request_sync, {}, crate::ffi_error::FfiError, $($rest)*);
    };
    (ffi_async, $($rest:tt)*) => {
        crate::database::macros::db_write_method_body!({#[uniffi::export]}, {async}, send_write_request_async, {.await}, crate::ffi_error::FfiError, $($rest)*);
    };
    (sync, $($rest:tt)*) => {
        crate::database::macros::db_write_method_body!({}, {}, send_write_request_sync, {}, anyhow::Error, $($rest)*);
    };
    (async, $($rest:tt)*) => {
        crate::database::macros::db_write_method_body!({}, {async}, send_write_request_async, {.await}, anyhow::Error, $($rest)*);
    };
}

macro_rules! db_read_modes {
    ($variant:ident, $ret:ty, $fields:tt, $mode:ident $client:ident $(, $($rest:tt)*)?) => {
        crate::database::macros::db_read_mode!($mode, $variant, $client, $ret, $fields);
        $( crate::database::macros::db_read_modes!($variant, $ret, $fields, $($rest)*); )?
    };
}

macro_rules! db_write_modes {
    ($variant:ident, $ret:ty, $fields:tt, $events:tt, $mode:ident $client:ident $(, $($rest:tt)*)?) => {
        crate::database::macros::db_write_mode!($mode, $variant, $client, $ret, $fields, $events);
        $( crate::database::macros::db_write_modes!($variant, $ret, $fields, $events, $($rest)*); )?
    };
}

macro_rules! db_requests {
    (
        reads {
            $( $rname:ident ( $($rfield:ident : $rty:ty),* $(,)? ) -> $rret:ty => $rfn:ident { $($rmode:ident $rclient:ident),+ $(,)? } ; )*
        }
        writes {
            $( $wname:ident ( $($wfield:ident : $wty:ty),* $(,)? ) -> $wret:ty => $wfn:ident { $($wmode:ident $wclient:ident),+ $(,)? } emits [ $($event:expr),* $(,)? ] ; )*
        }
    ) => {
        pub enum ReadRequest {
            $(
                $rname {
                    $($rfield: $rty,)*
                    reply: tokio::sync::oneshot::Sender<anyhow::Result<$rret>>,
                }
            ),*
        }

        pub enum WriteRequest {
            $(
                $wname {
                    $($wfield: $wty,)*
                    reply: tokio::sync::oneshot::Sender<anyhow::Result<$wret>>,
                }
            ),*
        }

        impl crate::database::workers::DbReader {
            pub fn dispatch(&mut self, request: ReadRequest) {
                match request {
                    $(
                        ReadRequest::$rname { $($rfield,)* reply } => {
                            let _ = reply.send(self.$rfn($(&$rfield),*));
                        }
                    ),*
                }
            }
        }

        impl crate::database::workers::DbWriter {
            pub fn dispatch(&mut self, request: WriteRequest) {
                match request {
                    $(
                        WriteRequest::$wname { $($wfield,)* reply } => {
                            let _ = reply.send(self.$wfn($($wfield),*));
                        }
                    ),*
                }
            }
        }

        $(
            crate::database::macros::db_read_modes!(
                $rname, $rret, ($($rfield: $rty),*), $($rmode $rclient),+
            );
        )*

        $(
            crate::database::macros::db_write_modes!(
                $wname, $wret, ($($wfield: $wty),*), [$($event),*], $($wmode $wclient),+
            );
        )*
    };
}

pub(crate) use db_read_method_body;
pub(crate) use db_read_mode;
pub(crate) use db_write_method_body;
pub(crate) use db_write_mode;
pub(crate) use db_read_modes;
pub(crate) use db_write_modes;
pub(crate) use db_requests;
