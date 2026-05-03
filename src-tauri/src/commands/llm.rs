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

    if let (Some(start), Some(end)) = (s.find('{'), s.rfind('}'))
        && end >= start
    {
        return s[start..=end].to_string();
    }

    s.to_string()
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
    fn falls_back_when_unparseable() {
        let raw = "totally not json";
        let out = parse_proofread_response(raw);
        assert_eq!(out.feedback, "totally not json");
        assert_eq!(out.key_expression, "");
        assert_eq!(out.example, "");
    }
}
