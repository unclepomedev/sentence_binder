CREATE TABLE IF NOT EXISTS sentences
(
    id TEXT PRIMARY KEY,
    original_text TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    source_context TEXT,
    audio_file_name TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS highlights
(
    id TEXT PRIMARY KEY,
    sentence_id TEXT NOT NULL,
    expression TEXT NOT NULL,
    meaning_and_usage TEXT,
    FOREIGN KEY (sentence_id) REFERENCES sentences (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_highlights_sentence_id ON highlights (
    sentence_id
);
