use crate::error::AppError;
use std::process::Command;
use tauri::command;

/// Plays the given text out loud using macOS's built-in TTS engine.
#[command]
pub async fn play_pronunciation(text: String) -> Result<(), AppError> {
    let output = Command::new("say")
        .arg(&text)
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to execute 'say' command: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AppError::Internal(format!("TTS failed: {}", err_msg)));
    }

    Ok(())
}

/// Instantly stops any currently playing macOS TTS audio.
#[command]
pub async fn stop_audio() -> Result<(), AppError> {
    let _ = Command::new("killall").arg("say").output();
    Ok(())
}
