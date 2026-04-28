mod capture;
mod commands;
mod constants;
mod credentials;
mod db;

use apple_native_keyring_store::keychain::Store as AppleKeychainStore;

use crate::commands::{delete_api_key, get_api_key, get_sentences, save_api_key, save_sentence};
use tauri::async_runtime::block_on;
use tauri::{generate_context, generate_handler, Builder, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = AppleKeychainStore::new().expect("Failed to initialize Apple Keychain store");
    keyring_core::set_default_store(store);

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
        .invoke_handler(generate_handler![
            save_sentence,
            get_sentences,
            save_api_key,
            get_api_key,
            delete_api_key
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}
