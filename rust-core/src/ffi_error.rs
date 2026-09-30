//! This module provides the error type used by UniFFI-exported functions.
//!
//! Most of the internal Rust code uses [`anyhow::Error`] for error handling.
//! `anyhow::Error` is a dynamic Rust error container and cannot be used
//! directly as a UniFFI-exposed error type. UniFFI requires exported errors
//! to be represented by a supported error type that can be serialized across
//! the FFI boundary.
//!
//! [`FfiError`] provides this boundary. Internal `anyhow::Error` values are
//! converted into [`FfiError::Internal`] with their formatted error context
//! stored as a `String`. This preserves the diagnostic information but loses
//! the original error types and their ability to be inspected or downcast
//! across the FFI boundary. If specific errors need to be handled across the
//! FFI, we can do this by adding additional variants to the FfiError enum. 
//! Otherwise, the main purpose is for diagnosing failures and this can be handled 
//! with the [`FfiError::Internal`] variant.

#[derive(uniffi::Error, Debug, PartialEq)]
pub enum FfiError {
    Internal { msg: String },
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiError::Internal { msg } => write!(f, "Rust core-lib error: {}", msg),
        }
    }
}

impl From<anyhow::Error> for FfiError {
    fn from(err: anyhow::Error) -> Self {
        FfiError::Internal {
            msg: format!("{err:#}"),
        }
    }
}

impl FfiError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal { msg: msg.into() }
    }
}

impl std::error::Error for FfiError {}
