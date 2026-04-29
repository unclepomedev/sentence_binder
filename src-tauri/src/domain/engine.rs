use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Network connection failed: {0}")]
    Network(String),
    #[error("Failed to parse LLM response: {0}")]
    Parse(String),
}

/// The core contract for any translation backend (Local MLX, OpenAI, DeepL, etc.)
pub trait LlmEngine: Send + Sync {
    /// Translates the captured text into Japanese.
    async fn translate(&self, text: &str) -> Result<String, LlmError>;

    /// Extracts the specific meaning and usage of a highlighted expression,
    /// taking the surrounding context into account.
    async fn extract_usage(&self, expression: &str, context: &str) -> Result<String, LlmError>;
}
