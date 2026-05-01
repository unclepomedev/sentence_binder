use crate::domain::models::Sentence;
use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

/// Safe chunk size for SQLite. (assert column_count * chunk_size <= 999)
const BULK_INSERT_CHUNK_SIZE: usize = 100;

/// sentences table
#[derive(Debug, sqlx::FromRow)]
struct SentenceRow {
    pub id: String,
    pub original_text: String,
    pub translated_text: String,
    pub source_context: Option<String>,
    /// milliseconds
    pub created_at: i64,
}

impl From<SentenceRow> for Sentence {
    fn from(row: SentenceRow) -> Self {
        Self {
            id: row.id,
            original_text: row.original_text,
            translated_text: row.translated_text,
            source_context: row.source_context,
            created_at: row.created_at,
        }
    }
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

/// Inserts multiple sentences with chunking of [`BULK_INSERT_CHUNK_SIZE`]
/// Skips duplicates based on the primary key `id`.
/// Returns the number of successfully inserted rows.
pub async fn insert_sentences_bulk(
    pool: &SqlitePool,
    sentences: &[Sentence],
) -> Result<usize, sqlx::Error> {
    if sentences.is_empty() {
        return Ok(0);
    }
    let mut total_inserted = 0;
    let mut tx = pool.begin().await?;

    for chunk in sentences.chunks(BULK_INSERT_CHUNK_SIZE) {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO sentences (id, original_text, translated_text, source_context, created_at) ",
        );
        query_builder.push_values(chunk, |mut b, sentence| {
            b.push_bind(&sentence.id)
                .push_bind(&sentence.original_text)
                .push_bind(&sentence.translated_text)
                .push_bind(&sentence.source_context)
                .push_bind(sentence.created_at);
        });
        query_builder.push(" ON CONFLICT(id) DO NOTHING");
        let query = query_builder.build();
        let result = query.execute(&mut *tx).await?;
        total_inserted += result.rows_affected() as usize;
    }

    tx.commit().await?;
    Ok(total_inserted)
}

pub async fn fetch_all_sentences(pool: &SqlitePool) -> Result<Vec<Sentence>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SentenceRow>(
        "SELECT id, original_text, translated_text, source_context, created_at
         FROM sentences
         ORDER BY created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await?;

    let sentences = rows.into_iter().map(Sentence::from).collect();
    Ok(sentences)
}

/// Updates the translated text and context of an existing sentence.
pub async fn update_translation(
    pool: &SqlitePool,
    id: &str,
    new_translation: &str,
    new_context: Option<&str>,
) -> Result<(), sqlx::Error> {
    let result =
        sqlx::query("UPDATE sentences SET translated_text = ?, source_context = ? WHERE id = ?")
            .bind(new_translation)
            .bind(new_context)
            .bind(id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(())
}

/// Deletes a sentence from the database by its ID.
pub async fn delete_sentence(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM sentences WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::slice;

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

    #[tokio::test]
    async fn test_update_translation() {
        let pool = setup_in_memory_db().await;

        let original = "I need an update.";
        let initial_translated = "古い翻訳";

        let sentence = insert_sentence(&pool, original, initial_translated, None)
            .await
            .expect("Failed to insert initial sentence");

        let new_translation = "新しい翻訳";
        let new_context = Some("Updated Context");

        let update_result =
            update_translation(&pool, &sentence.id, new_translation, new_context).await;
        assert!(update_result.is_ok());

        let row: (String, String, Option<String>) = sqlx::query_as(
            "SELECT original_text, translated_text, source_context FROM sentences WHERE id = ?",
        )
        .bind(&sentence.id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated row");

        assert_eq!(row.0, original, "Original text should remain unchanged");
        assert_eq!(row.1, new_translation, "Translated text should be updated");
        assert_eq!(
            row.2.as_deref(),
            new_context,
            "Source context should be updated"
        );
    }

    #[tokio::test]
    async fn test_update_translation_unknown_id_returns_error() {
        let pool = setup_in_memory_db().await;

        let unknown_id = Uuid::new_v4().to_string();
        let result = update_translation(&pool, &unknown_id, "any", None).await;

        assert!(
            matches!(result, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound for an unknown id, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_delete_sentence() {
        let pool = setup_in_memory_db().await;

        let sentence = insert_sentence(&pool, "Delete me", "私を削除して", None)
            .await
            .expect("Failed to insert sentence");

        let rows_before = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(rows_before.len(), 1);

        let delete_result = delete_sentence(&pool, &sentence.id).await;
        assert!(delete_result.is_ok());

        let rows_after = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(rows_after.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_sentence_unknown_id_returns_error() {
        let pool = setup_in_memory_db().await;
        let unknown_id = Uuid::new_v4().to_string();
        let result = delete_sentence(&pool, &unknown_id).await;
        assert!(
            matches!(result, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound for an unknown id, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_insert_sentences_bulk_empty() {
        let pool = setup_in_memory_db().await;
        let sentences: Vec<Sentence> = vec![];

        let result = insert_sentences_bulk(&pool, &sentences).await;
        assert!(result.is_ok(), "Empty bulk insert should return Ok");

        let fetched = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(fetched.len(), 0, "Database should remain empty");
    }

    #[tokio::test]
    async fn test_insert_sentences_bulk_small() {
        let pool = setup_in_memory_db().await;

        let sentences: Vec<Sentence> = (0..5)
            .map(|i| Sentence {
                id: Uuid::new_v4().to_string(),
                original_text: format!("Original {}", i),
                translated_text: format!("Translated {}", i),
                source_context: Some(format!("Context {}", i)),
                created_at: 1000 + i as i64,
            })
            .collect();

        let result = insert_sentences_bulk(&pool, &sentences).await;
        assert!(result.is_ok(), "Small bulk insert should succeed");

        let fetched = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(fetched.len(), 5, "All 5 sentences should be inserted");

        let found = fetched
            .iter()
            .find(|s| s.original_text == "Original 3")
            .unwrap();
        assert_eq!(found.translated_text, "Translated 3");
        assert_eq!(found.source_context.as_deref(), Some("Context 3"));
    }

    #[tokio::test]
    async fn test_insert_sentences_bulk_large_chunking() {
        let pool = setup_in_memory_db().await;
        let total_sentences = 250;

        let sentences: Vec<Sentence> = (0..total_sentences)
            .map(|i| Sentence {
                id: Uuid::new_v4().to_string(),
                original_text: format!("Bulk Original {}", i),
                translated_text: format!("Bulk Translated {}", i),
                source_context: None,
                created_at: Utc::now().timestamp_millis(),
            })
            .collect();

        let result = insert_sentences_bulk(&pool, &sentences).await;
        assert!(result.is_ok(), "Large chunked bulk insert should succeed");

        let fetched = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(
            fetched.len(),
            total_sentences as usize,
            "All 250 sentences should be inserted"
        );
    }

    #[tokio::test]
    async fn test_insert_sentences_bulk_skips_full_duplicates() {
        let pool = setup_in_memory_db().await;

        let sentence1 = Sentence {
            id: Uuid::new_v4().to_string(),
            original_text: "Existing 1".to_string(),
            translated_text: "Trans 1".to_string(),
            source_context: None,
            created_at: 1000,
        };
        let sentence2 = Sentence {
            id: Uuid::new_v4().to_string(),
            original_text: "Existing 2".to_string(),
            translated_text: "Trans 2".to_string(),
            source_context: None,
            created_at: 2000,
        };

        let batch = vec![sentence1.clone(), sentence2.clone()];

        let first_insert_count = insert_sentences_bulk(&pool, &batch).await.unwrap();
        assert_eq!(first_insert_count, 2, "First insert should add 2 rows");

        let second_insert_count = insert_sentences_bulk(&pool, &batch).await.unwrap();
        assert_eq!(
            second_insert_count, 0,
            "Second insert should skip all duplicates and return 0"
        );

        let fetched = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(
            fetched.len(),
            2,
            "Database should still only have 2 sentences total"
        );
    }

    #[tokio::test]
    async fn test_insert_sentences_bulk_partial_duplicates() {
        let pool = setup_in_memory_db().await;

        let sentence_old = Sentence {
            id: Uuid::new_v4().to_string(),
            original_text: "Old Sentence".to_string(),
            translated_text: "Old Trans".to_string(),
            source_context: None,
            created_at: 1000,
        };
        let sentence_new = Sentence {
            id: Uuid::new_v4().to_string(),
            original_text: "New Sentence".to_string(),
            translated_text: "New Trans".to_string(),
            source_context: None,
            created_at: 2000,
        };

        insert_sentences_bulk(&pool, slice::from_ref(&sentence_old))
            .await
            .unwrap();

        let mixed_batch = vec![sentence_old.clone(), sentence_new.clone()];

        let mixed_insert_count = insert_sentences_bulk(&pool, &mixed_batch).await.unwrap();
        assert_eq!(
            mixed_insert_count, 1,
            "Should insert 1 new sentence and skip 1 duplicate"
        );

        let fetched = fetch_all_sentences(&pool).await.unwrap();
        assert_eq!(fetched.len(), 2, "Database should have exactly 2 sentences");

        assert!(fetched.iter().any(|s| s.id == sentence_old.id));
        assert!(fetched.iter().any(|s| s.id == sentence_new.id));
    }
}
