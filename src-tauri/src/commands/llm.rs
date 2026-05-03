use crate::domain::engine::LlmEngine;
use crate::domain::models::ProofreadFeedback;
use crate::error::AppError;
use crate::infrastructure::mlx::{MlxConfig, MlxEngine};
use tauri::command;

/// Extracts usage and meaning using the local MLX server.
#[command]
pub async fn extract_usage(expression: String, context: String) -> Result<String, AppError> {
    let engine = MlxEngine::new(MlxConfig::default());

    let result = engine
        .extract_usage(&expression, &context)
        .await
        .map_err(|e| {
            eprintln!("[commands] Usage extraction failed: {}", e);
            e
        })?;

    Ok(result)
}

#[command]
pub async fn proofread_sentence(
    original_text: String,
    translated_text: String,
    user_attempt: String,
) -> Result<ProofreadFeedback, AppError> {
    let attempt = user_attempt.trim();
    if attempt.is_empty() {
        return Err(AppError::Validation(
            "Cannot proofread an empty attempt".to_string(),
        ));
    }

    let engine = MlxEngine::new(MlxConfig::default());

    let raw_response = engine
        .proofread_attempt(&original_text, &translated_text, attempt)
        .await
        .map_err(|e| {
            eprintln!("[commands] Proofread failed: {}", e);
            e
        })?;

    let feedback_obj = parse_proofread_response(&raw_response);

    Ok(feedback_obj)
}

/// Parses ProofreadFeedback from an LLM response.
/// LLMs frequently ignore JSON formatting constraints (e.g., wrapping in markdown
/// fences or adding conversational prose). This extracts the JSON payload and
/// provides a safe UI fallback if parsing completely fails.
fn parse_proofread_response(raw: &str) -> ProofreadFeedback {
    let cleaned = clean_llm_json(raw);

    match serde_json::from_str::<ProofreadFeedback>(&cleaned) {
        Ok(obj) => obj,
        Err(e) => {
            let total_len = raw.len();
            if cfg!(debug_assertions) {
                let snippet = truncate_for_log(raw, 500);
                eprintln!(
                    "[commands] Failed to parse LLM JSON: {} (raw_len={}) snippet: {}",
                    e, total_len, snippet
                );
            } else {
                eprintln!(
                    "[commands] Failed to parse LLM JSON: {} (raw_len={})",
                    e, total_len
                );
            }
            ProofreadFeedback {
                feedback: raw.trim().to_string(),
                key_expression: String::new(),
                example: String::new(),
            }
        }
    }
}

fn truncate_for_log(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let head: String = s.chars().take(max_chars).collect();
    format!("{}…[+{} chars omitted]", head, char_count - max_chars)
}

fn clean_llm_json(raw: &str) -> String {
    let mut s = raw.trim();

    if let Some(rest) = s.strip_prefix("```json") {
        s = rest.trim_start();
    } else if let Some(rest) = s.strip_prefix("```JSON") {
        s = rest.trim_start();
    } else if let Some(rest) = s.strip_prefix("```") {
        s = rest.trim_start();
    }

    if let Some(rest) = s.strip_suffix("```") {
        s = rest.trim_end();
    }

    if let Some(extracted) = extract_first_balanced_object(s) {
        return extracted;
    }

    s.to_string()
}

/// Scans `s` for the first balanced `{ ... }` JSON object, correctly handling
/// quoted strings (with escape sequences) so that braces inside strings do not
/// affect nesting. Returns `None` if no balanced object is found.
fn extract_first_balanced_object(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'{' {
            let start = i;
            let mut depth = 0i32;
            let mut in_string = false;
            let mut escape = false;
            let mut j = i;
            while j < len {
                let c = bytes[j];
                if in_string {
                    if escape {
                        escape = false;
                    } else if c == b'\\' {
                        escape = true;
                    } else if c == b'"' {
                        in_string = false;
                    }
                } else {
                    match c {
                        b'"' => in_string = true,
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(s[start..=j].to_string());
                            }
                        }
                        _ => {}
                    }
                }
                j += 1;
            }
            // Unbalanced from this `{`; stop scanning further opens since
            // the remainder is part of an unterminated object.
            return None;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_json() {
        let raw = r#"{"feedback":"good","key_expression":"hello","example":"Hello there."}"#;
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "good");
        assert_eq!(out.key_expression, "hello");
        assert_eq!(out.example, "Hello there.");
    }

    #[test]
    fn parses_json_in_code_fence() {
        let raw = "```json\n{\"feedback\":\"f\",\"key_expression\":\"k\",\"example\":\"e\"}\n```";
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "f");
        assert_eq!(out.key_expression, "k");
        assert_eq!(out.example, "e");
    }

    #[test]
    fn parses_json_with_surrounding_prose() {
        let raw = "Sure! Here is the result:\n{\"feedback\":\"f\",\"key_expression\":\"k\",\"example\":\"e\"}\nHope it helps.";
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "f");
    }

    #[test]
    fn parses_json_with_trailing_brace_in_prose() {
        // A stray `}` after the JSON object (e.g. from prose or a code-fence
        // mishap) must not be swallowed into the extracted JSON.
        let raw = "{\"feedback\":\"f\",\"key_expression\":\"k\",\"example\":\"e\"}\nNote: }";
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "f");
        assert_eq!(out.key_expression, "k");
        assert_eq!(out.example, "e");
    }

    #[test]
    fn ignores_braces_inside_strings() {
        let raw = r#"{"feedback":"use { and }","key_expression":"k","example":"e"}"#;
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "use { and }");
        assert_eq!(out.key_expression, "k");
        assert_eq!(out.example, "e");
    }

    #[test]
    fn parses_json_with_missing_optional_fields() {
        // Only `feedback` is present; `key_expression` and `example` are absent.
        // Must succeed (no raw-text fallback) with empty strings for missing fields.
        let raw = r#"{"feedback":"only feedback"}"#;
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "only feedback");
        assert_eq!(out.key_expression, "");
        assert_eq!(out.example, "");
    }

    #[test]
    fn parses_json_with_missing_example() {
        let raw = r#"{"feedback":"f","key_expression":"k"}"#;
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "f");
        assert_eq!(out.key_expression, "k");
        assert_eq!(out.example, "");
    }

    #[test]
    fn falls_back_when_unparseable() {
        let raw = "totally not json";
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "totally not json");
        assert_eq!(out.key_expression, "");
        assert_eq!(out.example, "");
    }
}
