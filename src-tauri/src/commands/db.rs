use crate::db;
use serde::Serialize;
use tauri::{command, State};

/// record's ID (created on insertion).
#[derive(Serialize)]
pub struct IdResponse {
    pub id: String,
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
    .map_err(|e| {
        eprintln!("[commands] Database error in save_sentence: {}", e);
        "Database error".to_string()
    })?;

    Ok(IdResponse { id })
}

/// Fetches all saved sentences from the database for the Library view.
#[command]
pub async fn get_sentences(state: State<'_, db::DbState>) -> Result<Vec<db::Sentence>, String> {
    db::fetch_all_sentences(&state.0).await.map_err(|e| {
        eprintln!("[commands] Database error in get_sentences: {}", e);
        "Database error".to_string()
    })
}
