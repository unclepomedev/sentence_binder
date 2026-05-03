use crate::domain::engine::{LlmEngine, LlmError};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

// prompt templates=================================================================================
const TRANSLATE_SYSTEM_PROMPT: &str = "You are a professional translator. Translate the following English text into natural, fluent Japanese. Provide ONLY the translation, without any explanations or conversational filler.";
const USAGE_SYSTEM_PROMPT: &str = "You are an English teacher. Explain the meaning and usage of the highlighted expression based on the provided context. Provide a concise explanation in Japanese and one clear example sentence in English. Output ONLY the explanation and example.";
const PROOFREAD_SYSTEM_PROMPT: &str = r#"You are an encouraging language tutor. The user is translating from Japanese to English.
You will receive the Japanese context, the target English sentence, and the user's attempt.

You MUST respond with a raw JSON object containing exactly these three keys:
{
  "feedback": "Briefly correct grammatical or nuance errors, or warmly validate if correct.",
  "key_expression": "One important English vocabulary word or idiom from the target sentence.",
  "example": "A brief, natural English example sentence using the key expression."
}
Do not output any markdown formatting, code blocks, or extra text outside this JSON object."#;
// -------------------------------------------------------------------------------------------------

const MLX_CLIENT_TIMEOUT_SECS: u64 = 60;

pub struct MlxConfig {
    pub endpoint: String,
    pub temperature: f32,
}

impl Default for MlxConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:8080/v1/chat/completions".to_string(),
            temperature: 0.3,
        }
    }
}

pub struct MlxEngine {
    client: Client,
    config: MlxConfig,
}

impl MlxEngine {
    pub fn new(config: MlxConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(MLX_CLIENT_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, config }
    }

    /// Sends a payload to the MLX server mimicking the OpenAI chat completions API.
    ///
    /// # Errors
    ///
    /// Returns an `LlmError::Network` if the local server at `self.config.endpoint`
    /// is unreachable, down, or returns an HTTP error.
    ///
    /// Returns an `LlmError::Parse` if the server returns an unexpected JSON structure.
    ///
    /// # Performance
    ///
    /// Local LLM inference time varies wildly based on model size and GPU memory pressure.
    /// This call blocks the async task until the full response is generated (streaming is
    /// not yet implemented).
    async fn send_chat_request(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, LlmError> {
        let payload = json!({
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "temperature": self.config.temperature
        });

        let response = self
            .client
            .post(&self.config.endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("Connection failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let snippet: String = body.chars().take(200).collect();
            eprintln!(
                "[mlx] LLM HTTP error: status={}, body_snippet={:?}",
                status, snippet
            );
            return Err(LlmError::Network(format!("HTTP {}", status)));
        }

        let json_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(format!("Parse failed: {}", e)))?;

        // `.pointer()` instead of `json_body["choices"][0]["message"]["content"]`:
        // to prevent instant panic during edge cases
        let content = json_body
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlmError::Parse("Missing content field in LLM response".to_string()))?
            .trim()
            .to_string();

        Ok(content)
    }
}

impl LlmEngine for MlxEngine {
    async fn translate(&self, text: &str) -> Result<String, LlmError> {
        self.send_chat_request(TRANSLATE_SYSTEM_PROMPT, text).await
    }

    async fn extract_usage(&self, expression: &str, context: &str) -> Result<String, LlmError> {
        let user_prompt = format!(
            "Context: {}\nExpression to explain: {}",
            context, expression
        );
        self.send_chat_request(USAGE_SYSTEM_PROMPT, &user_prompt)
            .await
    }

    async fn proofread_attempt(
        &self,
        original_text: &str,
        translated_text: &str,
        user_attempt: &str,
    ) -> Result<String, LlmError> {
        let user_prompt = format!(
            "Japanese context: {}\nCorrect English: {}\nUser's attempt: {}",
            translated_text, original_text, user_attempt
        );
        self.send_chat_request(PROOFREAD_SYSTEM_PROMPT, &user_prompt)
            .await
    }
}
// TODO: replace hardcoded English/Japanese with user-provided language settings

// ===============================================================================================
// Unit tests
// ===============================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_send_chat_request_success() {
        let mock_server = MockServer::start().await;

        let mock_response = json!({
            "choices": [{
                "message": {
                    "content": "This is the mocked AI response."
                }
            }]
        });

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        let config = MlxConfig {
            endpoint: mock_server.uri(),
            temperature: 0.3,
        };
        let engine = MlxEngine::new(config);

        let result = engine.send_chat_request("system", "user").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is the mocked AI response.");
    }

    #[tokio::test]
    async fn test_send_chat_request_handles_http_errors() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Model crashed"))
            .mount(&mock_server)
            .await;

        let config = MlxConfig {
            endpoint: mock_server.uri(),
            temperature: 0.3,
        };
        let engine = MlxEngine::new(config);

        let result = engine.send_chat_request("system", "user").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            LlmError::Network(msg) => assert!(msg.contains("HTTP 500")),
            _ => panic!("Expected Network error"),
        }
    }

    #[tokio::test]
    async fn test_send_chat_request_handles_malformed_json() {
        let mock_server = MockServer::start().await;

        let bad_json = json!({ "wrong_key": "data" });

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(bad_json))
            .mount(&mock_server)
            .await;

        let config = MlxConfig {
            endpoint: mock_server.uri(),
            temperature: 0.3,
        };
        let engine = MlxEngine::new(config);

        let result = engine.send_chat_request("system", "user").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            LlmError::Parse(msg) => assert!(msg.contains("Missing content field")),
            _ => panic!("Expected Parse error"),
        }
    }
}
