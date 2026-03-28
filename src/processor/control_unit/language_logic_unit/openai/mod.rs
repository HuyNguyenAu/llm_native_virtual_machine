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
    fn post_json<T: miniserde::Deserialize>(
        endpoint: &str,
        body: String,
        error_variant: fn(BaseException) -> Exception,
        context: &str,
    ) -> Result<T, Exception> {
        let url = format!("{}/{}", BASE_URL, endpoint);
        let response = post(&url).with_body(body).send().map_err(|e| {
            (error_variant)(BaseException::caused_by(
                format!("Failed to send {} request.", context),
                e,
            ))
        })?;

        if response.status_code != 200 {
            return Err((error_variant)(BaseException::new(
                format!(
                    "{} request failed with status {}: {}",
                    context, response.status_code, response.reason_phrase
                ),
                None,
            )));
        }

        let text = response.as_str().map_err(|e| {
            (error_variant)(BaseException::caused_by(
                format!("Failed to read {} response.", context),
                e,
            ))
        })?;

        from_str::<T>(text).map_err(|e| {
            (error_variant)(BaseException::caused_by(
                format!("Failed to deserialise {} response: {}", context, text),
                e,
            ))
        })
    }

    pub fn chat_completion(
        request: OpenAIChatCompletionRequest,
    ) -> Result<OpenAIChatCompletionResponse, Exception> {
        Self::post_json(
            CHAT_COMPLETION_ENDPOINT,
            json::to_string(&request),
            Exception::OpenAIChatCompletion,
            "chat",
        )
    }

    pub fn embeddings(
        request: OpenAIEmbeddingsRequest,
    ) -> Result<OpenAIEmbeddingsResponse, Exception> {
        Self::post_json(
            EMBEDDINGS_ENDPOINT,
            json::to_string(&request),
            Exception::OpenAIEmbeddings,
            "embedding",
        )
    }
}
