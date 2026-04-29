use crate::domain::engine::{LlmEngine, LlmError};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

// prompt templates=================================================================================
const TRANSLATE_SYSTEM_PROMPT: &str = "You are a professional translator. Translate the following English text into natural, fluent Japanese. Provide ONLY the translation, without any explanations or conversational filler.";

const USAGE_SYSTEM_PROMPT: &str = "You are an English teacher. Explain the meaning and usage of the highlighted expression based on the provided context. Provide a concise explanation in Japanese and one clear example sentence in English. Output ONLY the explanation and example.";
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
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Network(format!("HTTP {}: {}", status, text)));
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
}
