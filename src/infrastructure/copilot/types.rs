use serde::{Deserialize, Serialize};

// OAuth Device Flow

#[derive(Serialize)]
pub struct DeviceCodeRequest {
    pub client_id: String,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u64,
}

#[derive(Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub error: Option<String>,
}

// Copilot Token Exchange

#[derive(Deserialize)]
pub struct CopilotTokenResponse {
    pub token: String,
    pub expires_at: i64,
}

// Persisted auth

#[derive(Serialize, Deserialize)]
pub struct StoredAuth {
    pub oauth_token: String,
}

// Chat Completions API (OpenAI-compatible)

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub struct ChatChoice {
    pub message: ChatResponseMessage,
}

#[derive(Deserialize)]
pub struct ChatResponseMessage {
    pub content: Option<String>,
}
