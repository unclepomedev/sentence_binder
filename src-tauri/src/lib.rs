mod capture;
mod commands;
mod constants;
mod credentials;
mod db;
mod domain;
mod error;
mod infrastructure;

use apple_native_keyring_store::keychain::Store as AppleKeychainStore;

use crate::commands::{
    CredentialsState, delete_api_key, extract_usage, get_sentences, has_api_key, save_api_key,
    save_sentence, translate_text,
};
use tauri::async_runtime::block_on;
use tauri::{Builder, Manager, generate_context, generate_handler};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let credentials_available = match AppleKeychainStore::new() {
        Ok(store) => {
            keyring_core::set_default_store(store);
            true
        }
        Err(e) => {
            eprintln!(
                "[lib] Failed to initialize Apple Keychain store: {}. \
                Credential commands will return \"Keychain unavailable\".",
                e
            );
            false
        }
    };

    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let handle = app.handle().clone();

            // Block the main thread just long enough to initialize the DB and run migrations
            let pool = block_on(async move { db::init_db(&handle).await })?;

            app.manage(db::DbState(pool));
            app.manage(CredentialsState {
                available: credentials_available,
            });
            capture::setup_event_tap(app.handle().clone());
            Ok(())
        })
        .invoke_handler(generate_handler![
            save_sentence,
            get_sentences,
            save_api_key,
            has_api_key,
            delete_api_key,
            extract_usage,
            translate_text
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}
