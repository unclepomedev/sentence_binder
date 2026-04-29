use crate::domain::engine::LlmEngine;
use crate::error::AppError;
use crate::infrastructure::mlx::{MlxConfig, MlxEngine};
use tauri::command;

/// Translates text using the local MLX server.
#[command]
pub async fn translate_text(text: String) -> Result<String, AppError> {
    // For now, we instantiate the MLX engine directly to get it working.
    // (We will add routing for OpenAI/Anthropic based on user settings later).
    let engine = MlxEngine::new(MlxConfig::default());

    let result = engine.translate(&text).await.map_err(|e| {
        eprintln!("[commands] Translation failed: {}", e);
        e
    })?;
    Ok(result)
}

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
