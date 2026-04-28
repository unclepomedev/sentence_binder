use crate::credentials::{self, LlmProvider};
use crate::db;
use serde::Serialize;
use tauri::{command, State};

/// record's ID (created on insertion).
#[derive(Serialize)]
pub struct IdResponse {
    pub id: String,
}

/// an API key.
#[derive(Serialize)]
pub struct KeyResponse {
    pub key: String,
}

/// Saves the API key securely into the macOS Keychain for the specified provider.
#[command]
pub async fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let p = LlmProvider::from_str(&provider).ok_or("Invalid provider specified")?;
    credentials::save_key(p, &key)
}

/// Retrieves the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn get_api_key(provider: String) -> Result<KeyResponse, String> {
    let p = LlmProvider::from_str(&provider).ok_or("Invalid provider specified")?;
    let key = credentials::get_key(p)?;
    Ok(KeyResponse { key })
}

/// Deletes the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn delete_api_key(provider: String) -> Result<(), String> {
    let p = LlmProvider::from_str(&provider).ok_or("Invalid provider specified")?;
    credentials::delete_key(p)
}

/// Saves a newly captured sentence and its translation into the SQLite database.
#[command]
pub async fn save_sentence(
    state: State<'_, db::DbState>,
    original_text: String,
    translated_text: String,
    source_context: Option<String>,
) -> Result<IdResponse, String> {
    let id = db::insert_sentence(
        &state.0,
        &original_text,
        &translated_text,
        source_context.as_deref(),
    )
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(IdResponse { id })
}

/// Fetches all saved sentences from the database for the Library view.
#[command]
pub async fn get_sentences(state: State<'_, db::DbState>) -> Result<Vec<db::Sentence>, String> {
    db::fetch_all_sentences(&state.0)
        .await
        .map_err(|e| format!("Database error: {}", e))
}
