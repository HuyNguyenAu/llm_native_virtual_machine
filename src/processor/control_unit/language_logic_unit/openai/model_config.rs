#[derive(Debug)]
pub struct ModelTextConfig {
    pub stream: bool,
    pub return_progress: bool,
    pub model: String,
    pub reasoning_format: String,
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

#[derive(Debug)]
pub struct ModelEmbeddingsConfig {
    pub model: String,
    pub encoding_format: String,
}
