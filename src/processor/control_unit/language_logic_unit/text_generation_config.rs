use crate::config::TextModelOverrides;

pub struct TextGenerationConfig {
    pub text_model: String,
    pub text_model_overrides: TextModelOverrides,
    pub base_url: String,
    pub chat_completion_endpoint: String,
    pub timeout_secs: u64,
    pub debug_chat: bool,
}
