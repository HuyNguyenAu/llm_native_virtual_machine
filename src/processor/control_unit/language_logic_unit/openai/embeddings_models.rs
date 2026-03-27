use miniserde::{Deserialize, Serialize};

use super::model_config::ModelEmbeddingsConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIEmbeddingsRequest {
    pub model: String,
    pub input: String,
    pub encoding_format: String,
}

impl OpenAIEmbeddingsRequest {
    pub fn new(content: &str, config: ModelEmbeddingsConfig) -> Self {
        Self {
            model: config.model,
            input: content.to_string(),
            encoding_format: config.encoding_format,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIEmbeddingsResponseEmbedding {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIEmbeddingsResponse {
    pub object: String,
    pub data: Vec<OpenAIEmbeddingsResponseEmbedding>,
}
