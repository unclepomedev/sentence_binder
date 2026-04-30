use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

/// sentences table
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Sentence {
    pub id: String,
    pub original_text: String,
    pub translated_text: String,
    pub source_context: Option<String>,
    /// milliseconds
    pub created_at: i64,
}

/// Records a sentence and returns the record.
pub async fn insert_sentence(
    pool: &SqlitePool,
    original_text: &str,
    translated_text: &str,
    source_context: Option<&str>,
) -> Result<Sentence, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();

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

    Ok(Sentence {
        id,
        original_text: original_text.to_string(),
        translated_text: translated_text.to_string(),
        source_context: source_context.map(|s| s.to_string()),
        created_at: now,
    })
}

pub async fn fetch_all_sentences(pool: &SqlitePool) -> Result<Vec<Sentence>, sqlx::Error> {
    sqlx::query_as::<_, Sentence>(
        "SELECT id, original_text, translated_text, source_context, created_at
         FROM sentences
         ORDER BY created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    /// Inserts a sentence with a custom timestamp for deterministic sorting tests.
    async fn insert_sentence_at(
        pool: &SqlitePool,
        original_text: &str,
        translated_text: &str,
        source_context: Option<&str>,
        created_at: i64,
    ) -> Result<String, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO sentences (id, original_text, translated_text, source_context, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(original_text)
        .bind(translated_text)
        .bind(source_context)
        .bind(created_at)
        .execute(pool)
        .await?;
        Ok(id)
    }

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

        let sentence_result = insert_sentence(&pool, original, translated, context).await;

        assert!(sentence_result.is_ok());

        let sentence = sentence_result.unwrap();
        assert_eq!(sentence.id.len(), 36); // UUID string length

        // Verify the data was actually written to the DB
        let row: (String, String) =
            sqlx::query_as("SELECT original_text, translated_text FROM sentences WHERE id = ?")
                .bind(&sentence.id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch inserted row");

        assert_eq!(row.0, original);
        assert_eq!(row.1, translated);
    }

    #[tokio::test]
    async fn test_fetch_all_sentences() {
        let pool = setup_in_memory_db().await;

        insert_sentence_at(&pool, "First", "一つ目", None, 1_000)
            .await
            .unwrap();
        insert_sentence_at(&pool, "Second", "二つ目", Some("Context"), 2_000)
            .await
            .unwrap();

        let sentences = fetch_all_sentences(&pool)
            .await
            .expect("Failed to fetch sentences");

        assert_eq!(sentences.len(), 2);

        assert_eq!(sentences[0].original_text, "Second");
        assert_eq!(sentences[0].source_context.as_deref(), Some("Context"));

        assert_eq!(sentences[1].original_text, "First");
        assert_eq!(sentences[1].source_context, None);
    }
}
