use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sentence {
    pub id: String,
    pub original_text: String,
    pub translated_text: String,
    pub source_context: Option<String>,
    pub tags: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupPayload {
    /// The schema version of this backup format
    pub version: u32,
    /// When this backup was created (timestamp, milliseconds)
    pub exported_at: i64,
    /// The actual user data
    pub sentences: Vec<Sentence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofreadFeedback {
    pub feedback: String,
    #[serde(default)]
    pub key_expression: String,
    #[serde(default)]
    pub example: String,
}
