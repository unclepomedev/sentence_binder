mod capture;
mod constants;
mod db;

use tauri::async_runtime::block_on;
use tauri::{command, generate_context, generate_handler, Builder, Manager, State};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Block the main thread just long enough to initialize the DB and run migrations
            let pool = block_on(async move { db::init_db(&handle).await })?;

            app.manage(db::DbState(pool));
            capture::setup_event_tap(app.handle().clone());
            Ok(())
        })
        .invoke_handler(generate_handler![save_sentence, get_sentences])
        .run(generate_context!())
        .expect("error while running tauri application");
}

/// IPC command returns the record id (UUID).
#[command]
async fn save_sentence(
    state: State<'_, db::DbState>,
    original_text: String,
    translated_text: String,
    source_context: Option<String>,
) -> Result<String, String> {
    db::insert_sentence(
        &state.0,
        &original_text,
        &translated_text,
        source_context.as_deref(),
    )
    .await
    .map_err(|e| format!("Failed to save sentence: {}", e))
}

#[command]
async fn get_sentences(state: State<'_, db::DbState>) -> Result<Vec<db::Sentence>, String> {
    db::fetch_all_sentences(&state.0)
        .await
        .map_err(|e| format!("Failed to fetch sentences: {}", e))
}
