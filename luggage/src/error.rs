use std::env::VarError;

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum LuggageError {
    /// This error is the catchall for any non-specific error.
    #[doc(hidden)]
    #[error("Unknown error has occurred.")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, LuggageError>;

impl From<urn::Error> for LuggageError {
    fn from(_value: urn::Error) -> Self {
        return LuggageError::Unknown;
    }
}

impl From<serde_json::Error> for LuggageError {
    fn from(_value: serde_json::Error) -> Self {
        return LuggageError::Unknown;
    }
}

impl From<VarError> for LuggageError {
    fn from(_value: VarError) -> Self {
        return LuggageError::Unknown;
    }
}

impl From<std::io::Error> for LuggageError {
    fn from(_value: std::io::Error) -> Self {
        return LuggageError::Unknown;
    }
}

// impl From<anyhow::Error> for LuggageError {
//     fn from(_value: anyhow::Error) -> Self {
//         return LuggageError::Unknown;
//     }
// }
