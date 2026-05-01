use crate::db;
use crate::domain::engine::LlmEngine;
use crate::domain::models::Sentence;
use crate::error::AppError;
use crate::infrastructure::mlx::{MlxConfig, MlxEngine};
use std::fs;
use tauri::{AppHandle, State, command};
use tauri_plugin_dialog::DialogExt;

/// Saves a newly captured sentence into the SQLite database, performing translation simultaneously.
/// If translation fails, the translation string will be left blank without stopping the process.
#[command]
pub async fn save_sentence(
    state: State<'_, db::DbState>,
    original_text: String,
    source_context: Option<String>,
) -> Result<Sentence, AppError> {
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
pub async fn get_sentences(state: State<'_, db::DbState>) -> Result<Vec<Sentence>, AppError> {
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
    let id = id.trim().to_string();
    if id.is_empty() {
        return Err(AppError::Validation("invalid id".to_string()));
    }

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

/// Exports all sentences to a user-selected JSON file on their local disk.
/// When the user canceled the dialog, it returns `Ok(())`.
#[command]
pub async fn export_sentences_json(
    app: AppHandle,
    state: State<'_, db::DbState>,
) -> Result<(), AppError> {
    let sentences = db::fetch_all_sentences(&state.0).await.map_err(|e| {
        eprintln!("[commands] Database error in export_sentences_json: {}", e);
        AppError::Db(e)
    })?;

    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("sentences_backup.json")
        .blocking_save_file();

    let Some(path) = file_path else {
        return Ok(());
    };

    let path_buf = path
        .into_path()
        .map_err(|_| AppError::Validation("Unsupported file path format".into()))?;
    let json_string = serde_json::to_string_pretty(&sentences).map_err(|e| {
        eprintln!("[commands] JSON serialization error: {}", e);
        AppError::Internal(format!("Failed to stringify data: {}", e))
    })?;

    fs::write(path_buf, json_string).map_err(|e| {
        eprintln!("[commands] File write error: {}", e);
        AppError::Internal(format!("Failed to write file to disk: {}", e))
    })?;

    Ok(())
}

/// Imports sentences from a JSON file and inserts them into the database.
/// Returns the number of successfully inserted sentences.
/// When the user canceled the dialog, it returns `Ok(0)`.
#[command]
pub async fn import_sentences_json(
    app: AppHandle,
    state: State<'_, db::DbState>,
) -> Result<usize, AppError> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file();

    let Some(path) = file_path else {
        return Ok(0);
    };

    let path_buf = path
        .into_path()
        .map_err(|_| AppError::Validation("Unsupported file path format".into()))?;

    let file_contents = fs::read_to_string(path_buf).map_err(|e| {
        eprintln!("[commands] File read error: {}", e);
        AppError::Internal(format!("Failed to read file: {}", e))
    })?;

    let sentences: Vec<Sentence> = serde_json::from_str(&file_contents).map_err(|e| {
        eprintln!("[commands] JSON parsing error: {}", e);
        AppError::Validation(format!("Invalid JSON format: {}", e))
    })?;

    if sentences.is_empty() {
        return Ok(0);
    }

    let inserted_count = db::insert_sentences_bulk(&state.0, &sentences)
        .await
        .map_err(|e| {
            eprintln!("[commands] Database error in import_sentences_json: {}", e);
            AppError::Db(e)
        })?;

    Ok(inserted_count)
}
