use std::sync::Arc;

pub struct AppConfig {
    pub api_notion: String,
    pub api_telegram: String,
    pub api_gemini: String,
    pub chat_id: String,
}

pub struct AppState {
    pub config: AppConfig,
    pub http_client: reqwest::Client,
}
pub type SharedState = Arc<AppState>;
