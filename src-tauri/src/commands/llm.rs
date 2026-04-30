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
