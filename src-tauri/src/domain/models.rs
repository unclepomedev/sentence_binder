use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sentence {
    pub id: String,
    pub original_text: String,
    pub translated_text: String,
    pub source_context: Option<String>,
    pub created_at: i64,
}
