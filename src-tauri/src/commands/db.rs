use crate::db;
use crate::domain::engine::LlmEngine;
use crate::error::AppError;
use crate::infrastructure::mlx::{MlxConfig, MlxEngine};
use tauri::{State, command};

/// Saves a newly captured sentence into the SQLite database, performing translation simultaneously.
/// If translation fails, the translation string will be left blank without stopping the process.
#[command]
pub async fn save_sentence(
    state: State<'_, db::DbState>,
    original_text: String,
    source_context: Option<String>,
) -> Result<db::Sentence, AppError> {
    let original_text = original_text.trim().to_string();
    if original_text.is_empty() {
        return Err(AppError::Validation(
            "original_text cannot be empty".to_string(),
        ));
    }
    let engine = MlxEngine::new(MlxConfig::default());

    let translated_text = engine.translate(&original_text).await.unwrap_or_else(|e| {
        eprintln!(
            "[commands] Translation failed, saving original text anyway. Error: {}",
            e
        );
        String::new()
    });

    let new_sentence = db::insert_sentence(
        &state.0,
        &original_text,
        &translated_text,
        source_context.as_deref(),
    )
    .await
    .map_err(|e| {
        eprintln!("[commands] Database error in save_sentence: {}", e);
        AppError::Db(e)
    })?;

    Ok(new_sentence)
}

/// Fetches all saved sentences from the database for the Library view.
#[command]
pub async fn get_sentences(state: State<'_, db::DbState>) -> Result<Vec<db::Sentence>, AppError> {
    db::fetch_all_sentences(&state.0).await.map_err(|e| {
        eprintln!("[commands] Database error in get_sentences: {}", e);
        AppError::Db(e)
    })
}

/// Updates the translation and context for a specific sentence.
#[command]
pub async fn update_sentence_translation(
    state: State<'_, db::DbState>,
    id: String,
    new_translation: String,
    new_context: Option<String>,
) -> Result<(), AppError> {
    let new_translation = new_translation.trim().to_string();

    let new_context = new_context
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    db::update_translation(&state.0, &id, &new_translation, new_context)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                eprintln!(
                    "[commands] update_sentence_translation: sentence not found: {}",
                    id
                );
                AppError::NotFound(format!("Sentence not found: {}", id))
            }
            other => {
                eprintln!(
                    "[commands] Database error in update_sentence_translation: {}",
                    other
                );
                AppError::Db(other)
            }
        })?;

    Ok(())
}

/// Deletes a specific sentence from the database.
#[command]
pub async fn delete_sentence(state: State<'_, db::DbState>, id: String) -> Result<(), AppError> {
    db::delete_sentence(&state.0, &id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                eprintln!("[commands] delete_sentence: sentence not found: {}", id);
                AppError::NotFound(format!("Sentence not found: {}", id))
            }
            other => {
                eprintln!("[commands] Database error in delete_sentence: {}", other);
                AppError::Db(other)
            }
        })?;

    Ok(())
}
