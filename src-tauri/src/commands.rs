use crate::credentials::{self, LlmProvider};
use crate::{constants, db};
use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
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

/// Parses a provider identifier coming from the frontend into an [LlmProvider].
fn parse_provider(provider: &str) -> Result<LlmProvider, String> {
    LlmProvider::from_str(provider).ok_or_else(|| "Invalid provider specified".to_string())
}

/// Saves the API key securely into the macOS Keychain for the specified provider.
#[command]
pub async fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let p = parse_provider(&provider)?;
    spawn_blocking(move || credentials::save_key(p, &key))
        .await
        .map_err(|e| {
            eprintln!(
                "[commands] spawn_blocking join failed in save_api_key: {}",
                e
            );
            "Internal error".to_string()
        })?
}

/// Checks whether an API key exists in the Keychain for the specified provider,
/// without returning the secret value to the frontend.
///
/// Returns `Ok(true)` if the key exists, and `Ok(false)` if it is cleanly confirmed
/// to be missing. Genuine Keychain access failures (e.g., OS permission denied)
/// will return an `Err`.
#[command]
pub async fn has_api_key(provider: String) -> Result<bool, String> {
    let p = parse_provider(&provider)?;
    spawn_blocking(move || match credentials::get_key(p) {
        Ok(_) => Ok(true),
        Err(e) if e == constants::KEY_NOT_FOUND_MESSAGE => Ok(false),
        Err(e) => Err(e),
    })
    .await
    .map_err(|e| {
        eprintln!(
            "[commands] spawn_blocking join failed in has_api_key: {}",
            e
        );
        "Internal error".to_string()
    })?
}

/// Retrieves the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn get_api_key(provider: String) -> Result<KeyResponse, String> {
    let p = parse_provider(&provider)?;
    let key = spawn_blocking(move || credentials::get_key(p))
        .await
        .map_err(|e| {
            eprintln!(
                "[commands] spawn_blocking join failed in get_api_key: {}",
                e
            );
            "Internal error".to_string()
        })??;
    Ok(KeyResponse { key })
}

/// Deletes the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn delete_api_key(provider: String) -> Result<(), String> {
    let p = parse_provider(&provider)?;
    spawn_blocking(move || credentials::delete_key(p))
        .await
        .map_err(|e| {
            eprintln!(
                "[commands] spawn_blocking join failed in delete_api_key: {}",
                e
            );
            "Internal error".to_string()
        })?
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
