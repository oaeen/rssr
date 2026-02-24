use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("invalid llm config: {0}")]
    InvalidConfig(String),
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("server returned status {status}: {body}")]
    HttpStatus { status: u16, body: String },
    #[error("invalid llm response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone, Default)]
pub struct LlmService;

impl LlmService {
    pub fn name(&self) -> &'static str {
        "llm"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

pub async fn call_chat_completion(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, LlmError> {
    validate_config(config)?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs.max(5)))
        .build()?;
    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let response = client
        .post(endpoint)
        .bearer_auth(config.api_key.trim())
        .json(&serde_json::json!({
            "model": config.model.trim(),
            "temperature": 0.2,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ]
        }))
        .send()
        .await?;
    let status = response.status().as_u16();
    let body = response.text().await?;
    if status >= 400 {
        return Err(LlmError::HttpStatus { status, body });
    }

    let parsed: ChatCompletionResponse = serde_json::from_str(&body)
        .map_err(|error| LlmError::InvalidResponse(error.to_string()))?;
    let content = parsed
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            LlmError::InvalidResponse("missing choices[0].message.content".to_string())
        })?;
    Ok(content)
}

pub fn validate_config(config: &LlmConfig) -> Result<(), LlmError> {
    if config.base_url.trim().is_empty() {
        return Err(LlmError::InvalidConfig(
            "base_url cannot be empty".to_string(),
        ));
    }
    if !config.base_url.starts_with("http://") && !config.base_url.starts_with("https://") {
        return Err(LlmError::InvalidConfig(
            "base_url must start with http:// or https://".to_string(),
        ));
    }
    if config.api_key.trim().is_empty() {
        return Err(LlmError::InvalidConfig(
            "api_key cannot be empty".to_string(),
        ));
    }
    if config.model.trim().is_empty() {
        return Err(LlmError::InvalidConfig("model cannot be empty".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Json;
    use axum::http::HeaderMap;
    use axum::routing::post;
    use axum::Router;
    use serde_json::Value;

    #[test]
    fn validate_config_rejects_invalid_fields() {
        let config = LlmConfig {
            base_url: "localhost".to_string(),
            api_key: "".to_string(),
            model: "".to_string(),
            timeout_secs: 10,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    async fn chat_handler(headers: HeaderMap, Json(payload): Json<Value>) -> Json<Value> {
        let auth = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        let model = payload
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let content = if auth.starts_with("Bearer sk-test") && model == "deepseek-chat" {
            "ok"
        } else {
            "invalid"
        };
        Json(serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": content
                    }
                }
            ]
        }))
    }

    #[tokio::test]
    async fn call_chat_completion_openai_compatible_contract() {
        let app = Router::new().route("/chat/completions", post(chat_handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let addr = listener.local_addr().expect("local addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server should run");
        });

        let config = LlmConfig {
            base_url: format!("http://{addr}"),
            api_key: "sk-test-123".to_string(),
            model: "deepseek-chat".to_string(),
            timeout_secs: 10,
        };
        let result = call_chat_completion(&config, "system", "user")
            .await
            .expect("call should succeed");

        assert_eq!(result, "ok");
        server.abort();
    }
}
