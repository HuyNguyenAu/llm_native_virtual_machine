#[derive(Debug, Clone, Default)]
pub struct TextModelOverrides {
    pub stream: Option<bool>,
    pub return_progress: Option<bool>,
    pub reasoning_format: Option<String>,
    pub temperature: Option<f32>,
    pub dynatemp_range: Option<f32>,
    pub dynatemp_exponent: Option<f32>,
    pub top_k: Option<u32>,
    pub top_p: Option<f32>,
    pub min_p: Option<f32>,
    pub xtc_probability: Option<f32>,
    pub xtc_threshold: Option<f32>,
    pub typ_p: Option<f32>,
    pub max_tokens: Option<i32>,
    pub repeat_last_n: Option<u32>,
    pub repeat_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub dry_multiplier: Option<f32>,
    pub dry_base: Option<f32>,
    pub dry_allowed_length: Option<u32>,
    pub dry_penalty_last_n: Option<i32>,
    pub timings_per_token: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub text_model: String,
    pub embedding_model: String,
    pub text_model_overrides: TextModelOverrides,
    pub debug_build: bool,
    pub debug_run: bool,
    pub debug_chat: bool,
}
