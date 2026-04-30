mod sentences;

pub use sentences::{Sentence, fetch_all_sentences, insert_sentence, update_translation};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::error::Error;
use std::fs::create_dir_all;

use crate::constants;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

pub struct DbState(pub SqlitePool);

pub async fn init_db(app_handle: &AppHandle) -> Result<SqlitePool, Box<dyn Error>> {
    let app_dir = app_handle.path().app_data_dir()?;
    if !app_dir.exists() {
        create_dir_all(&app_dir)?;
    }

    let db_path = app_dir.join(constants::DB_NAME);
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(constants::MAX_DB_CONNECTIONS)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
