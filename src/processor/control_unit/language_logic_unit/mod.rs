use crate::{
    assembler::roles,
    exception::{BaseException, Exception},
    processor::{
        control_unit::language_logic_unit::openai::{
            OpenAIClient,
            chat_completion_models::{
                OpenAIChatCompletionRequest, OpenAIChatCompletionRequestText,
            },
            embeddings_models::OpenAIEmbeddingsRequest,
            model_config::{ModelEmbeddingsConfig, ModelTextConfig},
        },
        registers::ContextMessage,
    },
};

mod openai;

const SYSTEM_PROMPT: &str =
    "Provide exactly the requested output. Follow structural markers strictly.";

pub struct LanguageLogicUnit;

impl LanguageLogicUnit {
    fn default_text_model(model: &str) -> ModelTextConfig {
        ModelTextConfig {
            stream: false,
            return_progress: false,
            model: model.to_string(),
            reasoning_format: "auto".to_string(),
            temperature: 0.3,
            max_tokens: -1,
            dynatemp_range: 0.0,
            dynatemp_exponent: 1.0,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.15,
            xtc_probability: 0.0,
            xtc_threshold: 0.1,
            typ_p: 1.0,
            repeat_last_n: 64,
            repeat_penalty: 1.05,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            dry_multiplier: 0.0,
            dry_base: 1.75,
            dry_allowed_length: 2,
            dry_penalty_last_n: -1,
            samplers: vec![
                "penalties".to_string(),
                "dry".to_string(),
                "top_n_sigma".to_string(),
                "top_k".to_string(),
                "typ_p".to_string(),
                "top_p".to_string(),
                "min_p".to_string(),
                "xtc".to_string(),
                "temperature".to_string(),
            ],
            timings_per_token: false,
        }
    }

    fn default_embeddings_model(model: &str) -> ModelEmbeddingsConfig {
        ModelEmbeddingsConfig {
            model: model.to_string(),
            encoding_format: "float".to_string(),
        }
    }

    fn clean_string(value: &str) -> String {
        value.trim().replace("\n", "").to_string()
    }

    // Merge consecutive messages with the same role into a single message,
    // joining their content with a newline. This version is easier to follow:
    fn merge_messages_by_role(
        messages: &[OpenAIChatCompletionRequestText],
    ) -> Result<Vec<OpenAIChatCompletionRequestText>, Exception> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let mut merged_messages = Vec::<OpenAIChatCompletionRequestText>::new();
        let mut current = messages[0].clone();

        for message in &messages[1..] {
            if message.role == current.role {
                current.content.push('\n');
                current.content.push_str(&message.content);
            } else {
                merged_messages.push(current);

                current = message.clone();
            }
        }

        merged_messages.push(current);

        Ok(merged_messages)
    }

    // Message must always start with system role, and then followed by a user role. Assistant role can only be after a user role, and never at the end.
    // This is because the assistant role is meant to provide additional context to the model, and should not be the final message that
    // the model sees before generating a response. By enforcing this structure, we can ensure that the model receives a clear and consistent
    // input format, which can help improve the quality of the generated responses.
    fn validate_messages(messages: &[OpenAIChatCompletionRequestText]) -> Result<(), Exception> {
        if messages.len() < 2 {
            return Err(Exception::LanguageLogic(BaseException::new(
                "Messages must contain at least a system and a user message.".to_string(),
                None,
            )));
        }

        // Must start with system message and then user message.
        if messages[0].role != roles::SYSTEM_ROLE {
            return Err(Exception::LanguageLogic(BaseException::new(
                "The first message must be a system message.".to_string(),
                None,
            )));
        }

        if messages[1].role != roles::USER_ROLE {
            return Err(Exception::LanguageLogic(BaseException::new(
                "The second message must be a user message.".to_string(),
                None,
            )));
        }

        // Messages should strictly alternate: assistant -> user -> assistant -> ...
        // And the sequence must end on a user message (assistant message may never be last).
        let mut expected_role = roles::ASSISTANT_ROLE;

        for message in messages.iter().skip(2) {
            if message.role != expected_role {
                return Err(Exception::LanguageLogic(BaseException::new(
                    format!(
                        "Unexpected role '{}' in messages, expected '{}'.",
                        message.role, expected_role
                    ),
                    None,
                )));
            }

            // Swap expected role for next message.
            expected_role = if expected_role == roles::ASSISTANT_ROLE {
                roles::USER_ROLE
            } else {
                roles::ASSISTANT_ROLE
            };
        }

        let last_message = match messages.last() {
            Some(message) => message,
            None => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Messages cannot be empty.".to_string(),
                    None,
                )));
            }
        };

        if last_message.role != roles::USER_ROLE {
            return Err(Exception::LanguageLogic(BaseException::new(
                format!(
                    "Messages must end with a user message, but the last message has role '{}'.",
                    last_message.role
                ),
                None,
            )));
        }

        Ok(())
    }

    fn chat(
        content: &str,
        context: &[ContextMessage],
        text_model: &str,
    ) -> Result<String, Exception> {
        let model = Self::default_text_model(text_model);
        let messages = std::iter::once(OpenAIChatCompletionRequestText {
            role: roles::SYSTEM_ROLE.to_string(),
            content: SYSTEM_PROMPT.to_string(),
        })
        .chain(
            context
                .iter()
                .map(|message| OpenAIChatCompletionRequestText {
                    role: message.role.clone(),
                    content: message.content.clone(),
                }),
        )
        .chain(std::iter::once(OpenAIChatCompletionRequestText {
            role: roles::USER_ROLE.to_string(),
            content: content.to_string(),
        }))
        .collect::<Vec<OpenAIChatCompletionRequestText>>();

        let messages = match Self::merge_messages_by_role(&messages) {
            Ok(merged_messages) => merged_messages,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Failed to execute chat completion.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        match Self::validate_messages(&messages) {
            Ok(_) => {}
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Failed to execute chat completion.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let request = OpenAIChatCompletionRequest {
            messages,
            stream: model.stream,
            return_progress: model.return_progress,
            model: model.model.clone(),
            reasoning_format: model.reasoning_format.clone(),
            temperature: model.temperature,
            max_tokens: model.max_tokens,
            dynatemp_range: model.dynatemp_range,
            dynatemp_exponent: model.dynatemp_exponent,
            top_k: model.top_k,
            top_p: model.top_p,
            min_p: model.min_p,
            xtc_probability: model.xtc_probability,
            xtc_threshold: model.xtc_threshold,
            typ_p: model.typ_p,
            repeat_last_n: model.repeat_last_n,
            repeat_penalty: model.repeat_penalty,
            presence_penalty: model.presence_penalty,
            frequency_penalty: model.frequency_penalty,
            dry_multiplier: model.dry_multiplier,
            dry_base: model.dry_base,
            dry_allowed_length: model.dry_allowed_length,
            dry_penalty_last_n: model.dry_penalty_last_n,
            samplers: model.samplers.to_vec(),
            timings_per_token: model.timings_per_token,
        };
        let response = match OpenAIClient::chat_completion(request) {
            Ok(response) => response,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Failed to execute chat completion.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let choice = match response.choices.first() {
            Some(choice) => choice,
            None => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "No choices returned from client.".to_string(),
                    None,
                )));
            }
        };
        let result = Self::clean_string(&choice.message.content);

        Ok(result)
    }

    fn embeddings(content: &str, embedding_model: &str) -> Result<Vec<f32>, Exception> {
        let model = Self::default_embeddings_model(embedding_model);
        let request = OpenAIEmbeddingsRequest {
            model: model.model.to_string(),
            input: content.to_string(),
            encoding_format: model.encoding_format.to_string(),
        };
        let response = match OpenAIClient::embeddings(request) {
            Ok(response) => response,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Failed to get embeddings response from client.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let embeddings = match response.data.first() {
            Some(embedding) => embedding,
            None => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "No embeddings returned from client.".to_string(),
                    None,
                )));
            }
        };

        Ok(embeddings.embedding.to_owned())
    }

    pub fn cosine_similarity(
        value_a: &str,
        value_b: &str,
        embedding_model: &str,
    ) -> Result<u32, Exception> {
        let value_a_embeddings = match Self::embeddings(value_a, embedding_model) {
            Ok(embeddings) => embeddings,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    format!("Failed to get embedding for value a \"{}\".", value_a),
                    Some(Box::new(exception)),
                )));
            }
        };
        let value_b_embeddings = match Self::embeddings(value_b, embedding_model) {
            Ok(embeddings) => embeddings,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    format!("Failed to get embedding for value b \"{}\".", value_b),
                    Some(Box::new(exception)),
                )));
            }
        };

        // Compute cosine similarity.
        let dot_product: f32 = value_a_embeddings
            .iter()
            .zip(value_b_embeddings.iter())
            .map(|(a, b)| a * b)
            .sum();
        let x_euclidean_length: f32 = value_a_embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
        let y_euclidean_length: f32 = value_b_embeddings.iter().map(|y| y * y).sum::<f32>().sqrt();
        let similarity = dot_product / (x_euclidean_length * y_euclidean_length);
        let percentage_similarity = similarity.clamp(0.0, 1.0) * 100.0;

        Ok(percentage_similarity.round() as u32)
    }

    pub fn string(
        micro_prompt: &str,
        context: &[ContextMessage],
        text_model: &str,
    ) -> Result<String, Exception> {
        let result = match Self::chat(micro_prompt, context, text_model) {
            Ok(result) => result,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Failed to execute string operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        Ok(result)
    }

    pub fn boolean(
        micro_prompt: &str,
        true_values: &[&str],
        false_values: &[&str],
        context: &[ContextMessage],
        text_model: &str,
        embedding_model: &str,
    ) -> Result<u32, Exception> {
        let value = match Self::string(micro_prompt, context, text_model) {
            Ok(value) => value,
            Err(exception) => {
                return Err(Exception::LanguageLogic(BaseException::new(
                    "Failed to execute boolean operation.".to_string(),
                    Some(Box::new(exception)),
                )));
            }
        };

        let mut true_scores = Vec::<u32>::new();

        for true_value in true_values {
            match Self::cosine_similarity(
                &value.to_lowercase(),
                &true_value.to_lowercase(),
                embedding_model,
            ) {
                Ok(score) => true_scores.push(score),
                Err(exception) => {
                    return Err(Exception::LanguageLogic(BaseException::new(
                        format!(
                            "Failed to execute boolean operation for true value '{}'.",
                            true_value
                        ),
                        Some(Box::new(exception)),
                    )));
                }
            }
        }

        let mut false_scores = Vec::<u32>::new();

        for false_value in false_values {
            match Self::cosine_similarity(
                &value.to_lowercase(),
                &false_value.to_lowercase(),
                embedding_model,
            ) {
                Ok(score) => false_scores.push(score),
                Err(exception) => {
                    return Err(Exception::LanguageLogic(BaseException::new(
                        format!(
                            "Failed to execute boolean operation for false value '{}'.",
                            false_value
                        ),
                        Some(Box::new(exception)),
                    )));
                }
            }
        }

        let max_true_score = true_scores.into_iter().max().unwrap_or(0);
        let max_false_score = false_scores.into_iter().max().unwrap_or(0);

        if max_true_score > max_false_score {
            return Ok(100);
        }

        Ok(0)
    }
}
