use crate::credentials::{CredentialError, delete_key, has_key, save_key};
use crate::domain::provider::LlmProvider;
use crate::error::AppError;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tauri::async_runtime::spawn_blocking;
use tauri::{State, command};
use tokio::time::timeout;

/// Max duration for keychain operations. Accounts for legitimate OS TouchID/password
/// prompts, but bounds the wait at 8s to prevent UI deadlocks if the OS keychain daemon hangs.
const CREDENTIAL_OP_TIMEOUT: Duration = Duration::from_secs(8);
const CREDENTIAL_OP_TIMED_OUT: &str = "Credential operation timed out";

/// Circuit-breaker threshold for keychain timeouts. Prevents Tokio blocking-pool exhaustion
/// when the OS keychain daemon permanently hangs (since `spawn_blocking` tasks cannot be pre-empted).
/// Reset by any successful op.
const CIRCUIT_BREAKER_TIMEOUT_THRESHOLD: usize = 3;

/// Runtime flag set during startup if the OS keychain initialized successfully.
/// If false, credential commands safely short-circuit to prevent ad-hoc keyring errors.
pub struct CredentialsState {
    pub available: bool,
    /// Count of consecutive credential-op timeouts. Tripped open when it reaches
    /// `CIRCUIT_BREAKER_TIMEOUT_THRESHOLD`; cleared on any successful op.
    pub consecutive_timeouts: AtomicUsize,
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
    if !state.available {
        return Err(AppError::Credential(KEYCHAIN_UNAVAILABLE.to_string()));
    }
    if state.consecutive_timeouts.load(Ordering::Relaxed) >= CIRCUIT_BREAKER_TIMEOUT_THRESHOLD {
        // Circuit breaker is open: refuse new ops to protect the Tokio blocking pool.
        return Err(AppError::Credential(KEYCHAIN_UNAVAILABLE.to_string()));
    }
    Ok(())
}

/// Executes a credential operation on a blocking thread, standardizing
/// error logging and mapping for frontend consumption.
async fn run_credential_blocking<F, T, E>(
    state: &State<'_, CredentialsState>,
    op_name: &'static str,
    f: F,
) -> Result<T, AppError>
where
    F: FnOnce() -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Display + Send + 'static,
{
    let join_handle = spawn_blocking(f);
    // Abort the join handle on timeout. While `spawn_blocking` threads cannot be forcefully preempted,
    // aborting drops the awaiter. This ensures a late-resolving keychain success doesn't silently
    // mutate state after we've already returned a timeout error to the frontend.
    let abort_handle = join_handle.inner().abort_handle();
    let join_result = match timeout(CREDENTIAL_OP_TIMEOUT, join_handle).await {
        Ok(r) => r,
        Err(_) => {
            abort_handle.abort();
            let prev = state.consecutive_timeouts.fetch_add(1, Ordering::Relaxed);
            eprintln!(
                "[commands] {} credential operation timed out after {:?} (consecutive_timeouts={}); aborting task",
                op_name,
                CREDENTIAL_OP_TIMEOUT,
                prev + 1
            );
            if prev + 1 == CIRCUIT_BREAKER_TIMEOUT_THRESHOLD {
                eprintln!(
                    "[commands] credential circuit breaker tripped after {} consecutive timeouts; further ops will short-circuit until restart",
                    CIRCUIT_BREAKER_TIMEOUT_THRESHOLD
                );
            }
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
    let mapped = result.map_err(|e| {
        eprintln!("[commands] {} credential error: {}", op_name, e);
        AppError::Credential(CREDENTIAL_OP_FAILED.to_string())
    });
    // Any completed op (success OR a non-timeout error from the keychain itself)
    // proves the daemon is responsive, so reset the timeout streak.
    state.consecutive_timeouts.store(0, Ordering::Relaxed);
    mapped
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
    run_credential_blocking::<_, _, CredentialError>(&state, "save_api_key", move || {
        save_key(p, &key)
    })
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
    run_credential_blocking::<_, _, CredentialError>(&state, "has_api_key", move || has_key(p))
        .await
}

/// Deletes the API key from the macOS Keychain for the specified provider.
#[command]
pub async fn delete_api_key(
    state: State<'_, CredentialsState>,
    provider: String,
) -> Result<(), AppError> {
    ensure_available(&state)?;
    let p = parse_provider(&provider)?;
    run_credential_blocking::<_, _, CredentialError>(&state, "delete_api_key", move || {
        delete_key(p)
    })
    .await
}
