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
    pub tags: String,
    /// milliseconds
    pub created_at: i64,
}

impl From<SentenceRow> for Sentence {
    fn from(row: SentenceRow) -> Self {
        let tags = if row.tags.trim().is_empty() {
            vec![]
        } else {
            row.tags.split(',').map(|s| s.trim().to_string()).collect()
        };
        Self {
            id: row.id,
            original_text: row.original_text,
            translated_text: row.translated_text,
            source_context: row.source_context,
            tags,
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
    tags: &[String],
) -> Result<Sentence, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    let tags_joined = tags.join(",");

    sqlx::query(
        "INSERT INTO sentences (id, original_text, translated_text, source_context, tags, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(original_text)
    .bind(translated_text)
    .bind(source_context)
    .bind(&tags_joined)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Sentence {
        id,
        original_text: original_text.to_string(),
        translated_text: translated_text.to_string(),
        source_context: source_context.map(|s| s.to_string()),
        tags: tags.to_vec(),
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
            "INSERT INTO sentences (id, original_text, translated_text, source_context, tags, created_at) ",
        );
        query_builder.push_values(chunk, |mut b, sentence| {
            b.push_bind(&sentence.id)
                .push_bind(&sentence.original_text)
                .push_bind(&sentence.translated_text)
                .push_bind(&sentence.source_context)
                .push_bind(sentence.tags.join(","))
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
        "SELECT id, original_text, translated_text, source_context, tags, created_at
         FROM sentences
         ORDER BY created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await?;

    let sentences = rows.into_iter().map(Sentence::from).collect();
    Ok(sentences)
}

/// Updates the translated text, context, and tags of an existing sentence.
pub async fn update_translation(
    pool: &SqlitePool,
    id: &str,
    new_translation: &str,
    new_context: Option<&str>,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let tags_joined = tags.join(",");
    let result = sqlx::query(
        "UPDATE sentences SET translated_text = ?, source_context = ?, tags = ? WHERE id = ?",
    )
    .bind(new_translation)
    .bind(new_context)
    .bind(tags_joined)
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

/// Searches sentences (using SQLite FTS5).
/// Results are ordered by relevance (rank), then by newest.
pub async fn search_sentences(
    pool: &SqlitePool,
    search_query: &str,
) -> Result<Vec<Sentence>, sqlx::Error> {
    let query = search_query.trim();
    if query.is_empty() {
        return fetch_all_sentences(pool).await;
    }

    let fts_query = build_fts_query(query);

    // If the query only contained invalid empty tags (e.g., "tag:  "), fetch all
    if fts_query.is_empty() {
        return fetch_all_sentences(pool).await;
    }

    let rows = sqlx::query_as::<_, SentenceRow>(
        r#"
        SELECT s.id, s.original_text, s.translated_text, s.source_context, s.tags, s.created_at
        FROM sentences s
        JOIN sentences_fts f ON s.id = f.id
        WHERE sentences_fts MATCH ?
        ORDER BY f.rank, s.created_at DESC
        "#,
    )
    .bind(fts_query)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Sentence::from).collect())
}

/// Parses a raw search string into an FTS5 MATCH query.
/// Supports exact matches via quotes and tag filtering (e.g., `tag:"business trip"`).
fn build_fts_query(query: &str) -> String {
    let mut terms = Vec::new();
    let mut current_term = String::new();
    let mut in_quotes = false;

    for c in query.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' | '\n' if !in_quotes => {
                if !current_term.is_empty() {
                    terms.push(current_term.clone());
                    current_term.clear();
                }
            }
            _ => current_term.push(c),
        }
    }
    if !current_term.is_empty() {
        terms.push(current_term);
    }

    terms
        .into_iter()
        .filter_map(|term| {
            if let Some(tag_value) = term.strip_prefix("tag:") {
                let clean_val = tag_value.trim();
                if clean_val.is_empty() {
                    None
                } else {
                    Some(format!("tags:\"{}\"*", clean_val))
                }
            } else {
                let clean_val = term.trim();
                if clean_val.is_empty() {
                    None
                } else {
                    Some(format!("\"{}\"*", clean_val))
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

// ===============================================================================================
// Unit tests
// ===============================================================================================
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
        tags: &[String],
        created_at: i64,
    ) -> Result<String, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let tags_joined = tags.join(",");
        sqlx::query(
            "INSERT INTO sentences (id, original_text, translated_text, source_context, tags, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(original_text)
        .bind(translated_text)
        .bind(source_context)
        .bind(tags_joined)
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
        let tags = vec!["test".to_string(), "sample".to_string()];

        let sentence_result = insert_sentence(&pool, original, translated, context, &tags).await;

        assert!(sentence_result.is_ok());

        let sentence = sentence_result.unwrap();
        assert_eq!(sentence.id.len(), 36); // UUID string length
        assert_eq!(sentence.tags.len(), 2);

        // Verify the data was actually written to the DB
        let row: (String, String, String) = sqlx::query_as(
            "SELECT original_text, translated_text, tags FROM sentences WHERE id = ?",
        )
        .bind(&sentence.id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch inserted row");

        assert_eq!(row.0, original);
        assert_eq!(row.1, translated);
        assert_eq!(row.2, "test,sample");
    }

    #[tokio::test]
    async fn test_fetch_all_sentences() {
        let pool = setup_in_memory_db().await;

        insert_sentence_at(&pool, "First", "一つ目", None, &[], 1_000)
            .await
            .unwrap();
        insert_sentence_at(&pool, "Second", "二つ目", Some("Context"), &[], 2_000)
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

        let sentence = insert_sentence(&pool, original, initial_translated, None, &[])
            .await
            .expect("Failed to insert initial sentence");

        let new_translation = "新しい翻訳";
        let new_context = Some("Updated Context");

        let update_result =
            update_translation(&pool, &sentence.id, new_translation, new_context, &[]).await;
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
    async fn test_update_translation_with_tags() {
        let pool = setup_in_memory_db().await;

        let sentence = insert_sentence(&pool, "Base text", "基本テキスト", None, &[])
            .await
            .expect("Failed to insert initial sentence");

        let new_tags = vec!["tag1".to_string(), "tag2".to_string()];

        let update_result =
            update_translation(&pool, &sentence.id, "基本テキスト", None, &new_tags).await;
        assert!(update_result.is_ok());

        let row: (String, String) =
            sqlx::query_as("SELECT original_text, tags FROM sentences WHERE id = ?")
                .bind(&sentence.id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch updated row");

        assert_eq!(row.1, "tag1,tag2", "Tags should be successfully updated");
    }

    #[tokio::test]
    async fn test_update_translation_unknown_id_returns_error() {
        let pool = setup_in_memory_db().await;

        let unknown_id = Uuid::new_v4().to_string();
        let result = update_translation(&pool, &unknown_id, "any", None, &[]).await;

        assert!(
            matches!(result, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound for an unknown id, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_delete_sentence() {
        let pool = setup_in_memory_db().await;

        let sentence = insert_sentence(&pool, "Delete me", "私を削除して", None, &[])
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
                tags: vec![],
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
                tags: vec![],
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
            tags: vec![],
            created_at: 1000,
        };
        let sentence2 = Sentence {
            id: Uuid::new_v4().to_string(),
            original_text: "Existing 2".to_string(),
            translated_text: "Trans 2".to_string(),
            source_context: None,
            tags: vec![],
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
            tags: vec![],
            created_at: 1000,
        };
        let sentence_new = Sentence {
            id: Uuid::new_v4().to_string(),
            original_text: "New Sentence".to_string(),
            translated_text: "New Trans".to_string(),
            source_context: None,
            tags: vec![],
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

    #[tokio::test]
    async fn test_search_sentences_empty_query_returns_all() {
        let pool = setup_in_memory_db().await;

        insert_sentence(&pool, "First", "一つ目", None, &[])
            .await
            .unwrap();
        insert_sentence(&pool, "Second", "二つ目", None, &[])
            .await
            .unwrap();

        let results = search_sentences(&pool, "").await.expect("Search failed");
        let results_spaces = search_sentences(&pool, "   ").await.expect("Search failed");

        assert_eq!(results.len(), 2, "Empty string should return all sentences");
        assert_eq!(
            results_spaces.len(),
            2,
            "Whitespace-only string should return all sentences"
        );
    }

    #[tokio::test]
    async fn test_search_sentences_prefix_and_exact_match() {
        let pool = setup_in_memory_db().await;

        insert_sentence(
            &pool,
            "The quick brown fox",
            "素早い茶色のキツネ",
            None,
            &[],
        )
        .await
        .unwrap();
        insert_sentence(
            &pool,
            "A completely different string",
            "全く違う文字列",
            None,
            &[],
        )
        .await
        .unwrap();

        let exact = search_sentences(&pool, "brown").await.unwrap();
        assert_eq!(exact.len(), 1);
        assert_eq!(exact[0].original_text, "The quick brown fox");

        let prefix = search_sentences(&pool, "bro").await.unwrap();
        assert_eq!(prefix.len(), 1);
        assert_eq!(prefix[0].original_text, "The quick brown fox");

        let upper = search_sentences(&pool, "BROWN").await.unwrap();
        assert_eq!(upper.len(), 1);

        let none = search_sentences(&pool, "apple").await.unwrap();
        assert_eq!(none.len(), 0);
    }

    #[tokio::test]
    async fn test_search_sentences_multi_column() {
        let pool = setup_in_memory_db().await;

        insert_sentence(
            &pool,
            "Target in original",
            "Ignored",
            Some("Ignored context"),
            &[],
        )
        .await
        .unwrap();

        insert_sentence(
            &pool,
            "Ignored",
            "Target in translated",
            Some("Ignored context"),
            &[],
        )
        .await
        .unwrap();

        insert_sentence(&pool, "Ignored", "Ignored", Some("Target in context"), &[])
            .await
            .unwrap();

        insert_sentence(&pool, "Ignored", "Ignored", None, &["Target".to_string()])
            .await
            .unwrap();

        let results = search_sentences(&pool, "Target").await.unwrap();
        assert_eq!(
            results.len(),
            4,
            "Should match across original, translated, context, AND tags columns"
        );
    }

    #[tokio::test]
    async fn test_search_sentences_triggers_update_and_delete() {
        let pool = setup_in_memory_db().await;

        let sentence = insert_sentence(&pool, "Initial text", "初期テキスト", None, &[])
            .await
            .unwrap();

        let after_insert = search_sentences(&pool, "Initial").await.unwrap();
        assert_eq!(after_insert.len(), 1, "Insert trigger failed to index");

        update_translation(
            &pool,
            &sentence.id,
            "Updated translation",
            Some("New Context"),
            &[],
        )
        .await
        .unwrap();

        let search_old = search_sentences(&pool, "初期テキスト").await.unwrap();
        assert_eq!(
            search_old.len(),
            0,
            "Update trigger failed to remove old index"
        );

        let search_new = search_sentences(&pool, "Context").await.unwrap();
        assert_eq!(
            search_new.len(),
            1,
            "Update trigger failed to add new index"
        );

        delete_sentence(&pool, &sentence.id).await.unwrap();

        let after_delete = search_sentences(&pool, "Initial").await.unwrap();
        assert_eq!(
            after_delete.len(),
            0,
            "Delete trigger failed to clear index"
        );
    }

    #[tokio::test]
    async fn test_search_sentences_with_special_characters() {
        let pool = setup_in_memory_db().await;

        insert_sentence(
            &pool,
            "Check out this link: https://example.com/page?q=1",
            "リンクを見て：https://example.com/page?q=1",
            Some("file.txt"),
            &[],
        )
        .await
        .unwrap();

        insert_sentence(&pool, "Normal sentence here.", "普通", None, &[])
            .await
            .unwrap();

        let results_url = search_sentences(&pool, "https://")
            .await
            .expect("Search with https:// failed");
        assert_eq!(results_url.len(), 1);
        assert!(results_url[0].original_text.contains("https://example.com"));

        let results_domain = search_sentences(&pool, "example.com")
            .await
            .expect("Search with dot failed");
        assert_eq!(results_domain.len(), 1);

        let results_file = search_sentences(&pool, "file.txt")
            .await
            .expect("Search with extension failed");
        assert_eq!(results_file.len(), 1);

        let results_none = search_sentences(&pool, "https://google.com").await.unwrap();
        assert_eq!(results_none.len(), 0);
    }

    #[test]
    fn test_build_fts_query_standard_terms() {
        assert_eq!(build_fts_query("apple"), "\"apple\"*");
        assert_eq!(
            build_fts_query("apple banana"),
            "\"apple\"* AND \"banana\"*"
        );
    }

    #[test]
    fn test_build_fts_query_quoted_phrases() {
        assert_eq!(build_fts_query("\"apple banana\""), "\"apple banana\"*");
        assert_eq!(
            build_fts_query("start \"middle phrase\" end"),
            "\"start\"* AND \"middle phrase\"* AND \"end\"*"
        );
    }

    #[test]
    fn test_build_fts_query_tags() {
        assert_eq!(build_fts_query("tag:business"), "tags:\"business\"*");
        assert_eq!(
            build_fts_query("tag:\"business trip\""),
            "tags:\"business trip\"*"
        );
    }

    #[test]
    fn test_build_fts_query_mixed_and_empty() {
        // Mixed tags and terms
        assert_eq!(
            build_fts_query("apple tag:fruit \"red delicious\""),
            "\"apple\"* AND tags:\"fruit\"* AND \"red delicious\"*"
        );

        // Empty tags should be stripped out cleanly
        assert_eq!(build_fts_query("tag:"), "");
        assert_eq!(build_fts_query("apple tag:"), "\"apple\"*");
    }
}
