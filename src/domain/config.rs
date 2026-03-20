pub enum AiProvider {
    Copilot,
}

pub struct AiConfig {
    pub provider: AiProvider,
    pub model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Copilot,
            model: "gpt-4o".to_string(),
        }
    }
}
