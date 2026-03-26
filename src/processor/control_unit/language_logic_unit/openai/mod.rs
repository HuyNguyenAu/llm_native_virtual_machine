use miniserde::json::{self, from_str};
use minreq::post;

use crate::{
    exception::{BaseException, Exception},
    processor::control_unit::language_logic_unit::openai::{
        chat_completion_models::{OpenAIChatCompletionRequest, OpenAIChatCompletionResponse},
        embeddings_models::{OpenAIEmbeddingsRequest, OpenAIEmbeddingsResponse},
    },
};

pub mod chat_completion_models;
pub mod embeddings_models;
pub mod model_config;

const BASE_URL: &str = "http://127.0.0.1:8080";
const CHAT_COMPLETION_ENDPOINT: &str = "v1/chat/completions";
const EMBEDDINGS_ENDPOINT: &str = "v1/embeddings";

pub struct OpenAIClient;

impl OpenAIClient {
    pub fn chat_completion(
        request: OpenAIChatCompletionRequest,
    ) -> Result<OpenAIChatCompletionResponse, Exception> {
        let url = format!("{}/{}", BASE_URL, CHAT_COMPLETION_ENDPOINT);
        let body = json::to_string(&request);
        let response = post(&url).with_body(body).send().map_err(|e| {
            Exception::OpenAIChatCompletion(BaseException::new(
                "Failed to send chat request.".to_string(),
                Some(Box::new(e.into())),
            ))
        })?;

        if response.status_code != 200 {
            return Err(Exception::OpenAIChatCompletion(BaseException::new(
                format!(
                    "Chat request failed with status {}: {}",
                    response.status_code, response.reason_phrase
                ),
                None,
            )));
        }

        let text = response.as_str().map_err(|e| {
            Exception::OpenAIChatCompletion(BaseException::new(
                format!("Failed to read chat response: {}", e),
                Some(Box::new(e.into())),
            ))
        })?;

        from_str::<OpenAIChatCompletionResponse>(text).map_err(|e| {
            Exception::OpenAIChatCompletion(BaseException::new(
                format!("Failed to deserialise chat response: {}", text),
                Some(Box::new(e.into())),
            ))
        })
    }

    pub fn embeddings(
        request: OpenAIEmbeddingsRequest,
    ) -> Result<OpenAIEmbeddingsResponse, Exception> {
        let url = format!("{}/{}", BASE_URL, EMBEDDINGS_ENDPOINT);
        let body = json::to_string(&request);
        let response = post(&url).with_body(body).send().map_err(|e| {
            Exception::OpenAIEmbeddings(BaseException::new(
                "Failed to send embedding request.".to_string(),
                Some(Box::new(e.into())),
            ))
        })?;

        if response.status_code != 200 {
            return Err(Exception::OpenAIEmbeddings(BaseException::new(
                format!(
                    "Embedding request failed with status {}.",
                    response.status_code
                ),
                None,
            )));
        }

        let text = response.as_str().map_err(|e| {
            Exception::OpenAIEmbeddings(BaseException::new(
                format!("Failed to read embedding response: {}", e),
                Some(Box::new(e.into())),
            ))
        })?;

        from_str::<OpenAIEmbeddingsResponse>(text).map_err(|e| {
            Exception::OpenAIEmbeddings(BaseException::new(
                format!("Failed to deserialise embedding response: {}", text),
                Some(Box::new(e.into())),
            ))
        })
    }
}
