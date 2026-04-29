use crate::domain::engine::LlmError;
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    /// Errors originating from the LLM Infrastructure (Network, Parsing, etc.)
    #[error("LLM Error: {0}")]
    Llm(#[from] LlmError),

    /// Errors originating from Database operations
    #[error("Database Error: {0}")]
    Db(#[from] sqlx::Error),

    /// Errors originating from the OS Keychain
    #[error("Credential Error: {0}")]
    Credential(String),

    /// Errors originating from user input validation
    #[error("Validation Error: {0}")]
    Validation(String),

    /// General fallback for unexpected system failures
    #[error("Internal Error: {0}")]
    Internal(String),
}

// Keep the Serialize implementation so Tauri can send these to React seamlessly
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
