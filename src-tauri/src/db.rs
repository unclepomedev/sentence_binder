use chrono::Utc;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use std::error::Error;
use std::fs::create_dir_all;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub const DB_NAME: &str = "sentence_binder.db";

pub struct DbState(pub SqlitePool);

pub async fn init_db(app_handle: &AppHandle) -> Result<SqlitePool, Box<dyn Error>> {
    let app_dir = app_handle.path().app_data_dir()?;
    if !app_dir.exists() {
        create_dir_all(&app_dir)?;
    }

    let db_path = app_dir.join(DB_NAME);
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Records a sentence and returns its UUID.
pub async fn insert_sentence(
    pool: &SqlitePool,
    original_text: &str,
    translated_text: &str,
    source_context: Option<&str>,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO sentences (id, original_text, translated_text, source_context, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(original_text)
    .bind(translated_text)
    .bind(source_context)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    // Helper to spin up an in-memory DB for tests
    async fn setup_in_memory_db() -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to create in-memory database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_insert_sentence() {
        let pool = setup_in_memory_db().await;

        let original = "This is a test.";
        let translated = "これはテストです。";
        let context = Some("Google Chrome");

        let id_result = insert_sentence(&pool, original, translated, context).await;

        assert!(id_result.is_ok());
        let id = id_result.unwrap();
        assert_eq!(id.len(), 36); // UUID string length

        // Verify the data was actually written to the DB
        let row: (String, String) =
            sqlx::query_as("SELECT original_text, translated_text FROM sentences WHERE id = ?")
                .bind(&id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch inserted row");

        assert_eq!(row.0, original);
        assert_eq!(row.1, translated);
    }
}
