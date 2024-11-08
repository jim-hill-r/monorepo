#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum LuggageError {
    /// This error is the catchall for any non-specific error.
    #[doc(hidden)]
    #[error("Unknown error has occurred.")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, LuggageError>;
