CREATE VIRTUAL TABLE sentences_fts USING fts5 (
id UNINDEXED,
original_text,
translated_text,
source_context
) ;

-- Backfill
INSERT INTO sentences_fts (id, original_text, translated_text, source_context)
SELECT id, original_text, translated_text, source_context FROM sentences ;

CREATE TRIGGER sentences_fts_insert AFTER INSERT ON sentences BEGIN
INSERT INTO sentences_fts (id, original_text, translated_text, source_context)
VALUES (new.id, new.original_text, new.translated_text, new.source_context) ;
END ;

CREATE TRIGGER sentences_fts_delete AFTER DELETE ON sentences BEGIN
DELETE FROM sentences_fts WHERE id = old.id ;
END ;

CREATE TRIGGER sentences_fts_update AFTER UPDATE ON sentences BEGIN
DELETE FROM sentences_fts WHERE id = old.id ;
INSERT INTO sentences_fts (id, original_text, translated_text, source_context)
VALUES (new.id, new.original_text, new.translated_text, new.source_context) ;
END ;
