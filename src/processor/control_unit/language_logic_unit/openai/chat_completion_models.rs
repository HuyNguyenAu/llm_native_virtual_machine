use miniserde::{Deserialize, Serialize};

use super::model_config::ModelTextConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIChatCompletionRequestText {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIChatCompletionRequest {
    pub messages: Vec<OpenAIChatCompletionRequestText>,
    pub stream: bool,
    pub return_progress: bool,
    pub reasoning_format: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: i32,
    pub dynatemp_range: f32,
    pub dynatemp_exponent: f32,
    pub top_k: u32,
    pub top_p: f32,
    pub min_p: f32,
    pub xtc_probability: f32,
    pub xtc_threshold: f32,
    pub typ_p: f32,
    pub repeat_last_n: u32,
    pub repeat_penalty: f32,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub dry_multiplier: f32,
    pub dry_base: f32,
    pub dry_allowed_length: u32,
    pub dry_penalty_last_n: i32,
    pub samplers: Vec<String>,
    pub timings_per_token: bool,
}

impl OpenAIChatCompletionRequest {
    pub fn new(messages: Vec<OpenAIChatCompletionRequestText>, config: ModelTextConfig) -> Self {
        Self {
            messages,
            stream: config.stream,
            return_progress: config.return_progress,
            model: config.model,
            reasoning_format: config.reasoning_format,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            dynatemp_range: config.dynatemp_range,
            dynatemp_exponent: config.dynatemp_exponent,
            top_k: config.top_k,
            top_p: config.top_p,
            min_p: config.min_p,
            xtc_probability: config.xtc_probability,
            xtc_threshold: config.xtc_threshold,
            typ_p: config.typ_p,
            repeat_last_n: config.repeat_last_n,
            repeat_penalty: config.repeat_penalty,
            presence_penalty: config.presence_penalty,
            frequency_penalty: config.frequency_penalty,
            dry_multiplier: config.dry_multiplier,
            dry_base: config.dry_base,
            dry_allowed_length: config.dry_allowed_length,
            dry_penalty_last_n: config.dry_penalty_last_n,
            samplers: config.samplers,
            timings_per_token: config.timings_per_token,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIChatCompletionResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIChatCompletionResponseChoice {
    pub index: u8,
    pub message: OpenAIChatCompletionResponseMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIChatCompletionResponse {
    pub model: String,
    pub choices: Vec<OpenAIChatCompletionResponseChoice>,
}
