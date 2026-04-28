use crate::credentials::{self, LlmProvider};
use crate::db;
use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::{command, State};

/// Runtime flag indicating whether credential-related functionality is
/// available. Set during startup based on whether the OS keychain store
/// initialized successfully. When `available` is `false`, every credential
/// command short-circuits with a uniform "Keychain unavailable" error so
/// the frontend doesn't see ad-hoc keyring failure messages.
pub struct CredentialsState {
    pub available: bool,
}

const KEYCHAIN_UNAVAILABLE: &str = "Keychain unavailable";

/// record's ID (created on insertion).
#[derive(Serialize)]
pub struct IdResponse {
    pub id: String,
}

/// Parses a provider identifier coming from the frontend into an [LlmProvider].
fn parse_provider(provider: &str) -> Result<LlmProvider, String> {
    LlmProvider::from_str(provider).ok_or_else(|| "Invalid provider specified".to_string())
}

fn ensure_available(state: &State<'_, CredentialsState>) -> Result<(), String> {
    if state.available {
        Ok(())
    } else {
        Err(KEYCHAIN_UNAVAILABLE.to_string())
    }
}

/// Saves the API key securely into the macOS Keychain for the specified provider.
#[command]
pub async fn save_api_key(
    state: State<'_, CredentialsState>,
    provider: String,
    key: String,
) -> Result<(), String> {
    ensure_available(&state)?;
    let p = parse_provider(&provider)?;
    let result = spawn_blocking(move || credentials::save_key(p, &key))
        .await
        .map_err(|e| {
            eprintln!(
                "[commands] spawn_blocking join failed in save_api_key: {}",
                e
            );
            "Internal error".to_string()
        })?;
    result.map_err(|e| {
        eprintln!("[commands] save_api_key credential error: {}", e);
        "Credential operation failed".to_string()
    })
}

/// Checks whether an API key exists in the Keychain for the specified provider,
/// without returning the secret value to the frontend.
///
/// Returns `Ok(true)` if the key exists, and `Ok(false)` if it is cleanly confirmed
/// to be missing. Genuine Keychain access failures (e.g., OS permission denied)
/// will return an `Err`.
#[command]
pub async fn has_api_key(
    state: State<'_, CredentialsState>,
    provider: String,
) -> Result<bool, String> {
    ensure_available(&state)?;
    let p = parse_provider(&provider)?;
    let result = spawn_blocking(move || credentials::has_key(p))
        .await
        .map_err(|e| {
            eprintln!(
                "[commands] spawn_blocking join failed in has_api_key: {}",
                e
            );
            "Internal error".to_string()
        })?;
    result.map_err(|e| {
        eprintln!("[commands] has_api_key credential error: {}", e);
        "Credential operation failed".to_string()
    })
}

/// Deletes the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn delete_api_key(
    state: State<'_, CredentialsState>,
    provider: String,
) -> Result<(), String> {
    ensure_available(&state)?;
    let p = parse_provider(&provider)?;
    let result = spawn_blocking(move || credentials::delete_key(p))
        .await
        .map_err(|e| {
            eprintln!(
                "[commands] spawn_blocking join failed in delete_api_key: {}",
                e
            );
            "Internal error".to_string()
        })?;
    result.map_err(|e| {
        eprintln!("[commands] delete_api_key credential error: {}", e);
        "Credential operation failed".to_string()
    })
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
