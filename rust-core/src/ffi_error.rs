#[derive(uniffi::Error, Debug, PartialEq)]
pub enum FfiError {
    Internal { msg: String },
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiError::Internal { msg } => write!(f, "Internal error: {}", msg),
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
