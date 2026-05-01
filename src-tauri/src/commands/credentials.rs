use crate::credentials::{CredentialError, delete_key, has_key, save_key};
use crate::domain::provider::LlmProvider;
use crate::error::AppError;
use std::fmt::Display;
use std::time::Duration;
use tauri::async_runtime::spawn_blocking;
use tauri::{State, command};
use tokio::time::timeout;

/// Max duration for keychain operations. Accounts for legitimate OS TouchID/password
/// prompts, but bounds the wait at 8s to prevent UI deadlocks if the OS keychain daemon hangs.
const CREDENTIAL_OP_TIMEOUT: Duration = Duration::from_secs(8);
const CREDENTIAL_OP_TIMED_OUT: &str = "Credential operation timed out";

/// Runtime flag set during startup if the OS keychain initialized successfully.
/// If false, credential commands safely short-circuit to prevent ad-hoc keyring errors.
pub struct CredentialsState {
    pub available: bool,
}

// Error messages for the frontend to display to the user. ==================
const KEYCHAIN_UNAVAILABLE: &str = "Keychain unavailable";
const CREDENTIAL_OP_FAILED: &str = "Credential operation failed";
const INTERNAL_ERROR: &str = "Internal error";
const API_KEY_EMPTY: &str = "API key cannot be empty";
const INVALID_PROVIDER: &str = "Invalid provider specified";
// --------------------------------------------------------------------------

/// Parses a provider identifier coming from the frontend into an [LlmProvider].
fn parse_provider(provider: &str) -> Result<LlmProvider, AppError> {
    LlmProvider::from_str(provider)
        .ok_or_else(|| AppError::Validation(INVALID_PROVIDER.to_string()))
}

fn ensure_available(state: &State<'_, CredentialsState>) -> Result<(), AppError> {
    if state.available {
        Ok(())
    } else {
        Err(AppError::Credential(KEYCHAIN_UNAVAILABLE.to_string()))
    }
}

/// Executes a credential operation on a blocking thread, standardizing
/// error logging and mapping for frontend consumption.
async fn run_credential_blocking<F, T, E>(op_name: &'static str, f: F) -> Result<T, AppError>
where
    F: FnOnce() -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Display + Send + 'static,
{
    let join_handle = spawn_blocking(f);
    let join_result = match timeout(CREDENTIAL_OP_TIMEOUT, join_handle).await {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "[commands] {} credential operation timed out after {:?}",
                op_name, CREDENTIAL_OP_TIMEOUT
            );
            return Err(AppError::Credential(CREDENTIAL_OP_TIMED_OUT.to_string()));
        }
    };
    let result = join_result.map_err(|e| {
        eprintln!(
            "[commands] spawn_blocking join failed in {}: {}",
            op_name, e
        );
        AppError::Internal(INTERNAL_ERROR.to_string())
    })?;
    result.map_err(|e| {
        eprintln!("[commands] {} credential error: {}", op_name, e);
        AppError::Credential(CREDENTIAL_OP_FAILED.to_string())
    })
}

/// Saves the API key securely into the macOS Keychain for the specified provider.
#[command]
pub async fn save_api_key(
    state: State<'_, CredentialsState>,
    provider: String,
    key: String,
) -> Result<(), AppError> {
    ensure_available(&state)?;

    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(API_KEY_EMPTY.to_string()));
    }
    let key = trimmed.to_string();
    let p = parse_provider(&provider)?;
    run_credential_blocking::<_, _, CredentialError>("save_api_key", move || save_key(p, &key))
        .await
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
) -> Result<bool, AppError> {
    ensure_available(&state)?;
    let p = parse_provider(&provider)?;
    run_credential_blocking::<_, _, CredentialError>("has_api_key", move || has_key(p)).await
}

/// Deletes the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn delete_api_key(
    state: State<'_, CredentialsState>,
    provider: String,
) -> Result<(), AppError> {
    ensure_available(&state)?;
    let p = parse_provider(&provider)?;
    run_credential_blocking::<_, _, CredentialError>("delete_api_key", move || delete_key(p)).await
}
