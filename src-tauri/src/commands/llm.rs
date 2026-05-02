use crate::domain::engine::LlmEngine;
use crate::error::AppError;
use crate::infrastructure::mlx::{MlxConfig, MlxEngine};
use tauri::command;

/// Extracts usage and meaning using the local MLX server.
#[command]
pub async fn extract_usage(expression: String, context: String) -> Result<String, AppError> {
    let engine = MlxEngine::new(MlxConfig::default());

    let result = engine
        .extract_usage(&expression, &context)
        .await
        .map_err(|e| {
            eprintln!("[commands] Usage extraction failed: {}", e);
            e
        })?;

    Ok(result)
}

#[command]
pub async fn proofread_sentence(
    original_text: String,
    translated_text: String,
    user_attempt: String,
) -> Result<String, AppError> {
    let attempt = user_attempt.trim();
    if attempt.is_empty() {
        return Err(AppError::Validation(
            "Cannot proofread an empty attempt".to_string(),
        ));
    }

    let engine = MlxEngine::new(MlxConfig::default());

    let feedback = engine
        .proofread_attempt(&original_text, &translated_text, attempt)
        .await
        .map_err(|e| {
            eprintln!("[commands] Proofread failed: {}", e);
            AppError::Internal("Failed to proofread".to_string())
        })?;

    Ok(feedback)
}
