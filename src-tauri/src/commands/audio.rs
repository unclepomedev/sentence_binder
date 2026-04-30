use crate::error::AppError;
use std::process::Command;
use tauri::async_runtime::spawn_blocking;
use tauri::command;

/// Plays the given text out loud using macOS's built-in TTS engine.
#[command]
pub async fn play_pronunciation(text: String) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::Validation(
            "Pronunciation text cannot be empty".to_string(),
        ));
    }
    let output = spawn_blocking(move || Command::new("say").arg(text).output())
        .await
        .map_err(|e| AppError::Internal(format!("TTS task join failed: {}", e)))?
        .map_err(|e| AppError::Internal(format!("Failed to execute 'say' command: {}", e)))?;

    // If `say` was terminated by a signal (e.g. SIGTERM from `killall say`
    // when the user clicked the stop button), `status.code()` is `None` on
    // Unix. That's an intentional cancellation, not a failure, so don't
    // surface it as an error to the UI.
    if !output.status.success() && output.status.code().is_some() {
        let err_msg = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AppError::Internal(format!("TTS failed: {}", err_msg)));
    }

    Ok(())
}

/// Instantly stops any currently playing macOS TTS audio.
///
/// `killall say` is a best-effort operation: if no `say` process is currently
/// running (because playback already finished, or because the user clicked
/// stop a moment too late), `killall` exits with status 1. That is not an
/// actual failure from the user's perspective, so we treat any non-zero exit
/// status as a no-op rather than surfacing it to the UI.
#[command]
pub async fn stop_audio() -> Result<(), AppError> {
    let result = spawn_blocking(|| Command::new("killall").arg("say").output())
        .await
        .map_err(|e| AppError::Internal(format!("Stop-audio task join failed: {}", e)))?;

    // If we couldn't even spawn `killall`, something is genuinely wrong with
    // the environment and we should report it. Otherwise, succeed regardless
    // of `killall`'s exit status — there is simply nothing more to stop.
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to execute 'killall': {}",
            e
        ))),
    }
}
