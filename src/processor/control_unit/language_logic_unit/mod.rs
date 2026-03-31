use crate::{
    assembler::roles,
    config::TextModelOverrides,
    constants::SYSTEM_PROMPT,
    exception::{BaseException, Exception},
    processor::{
        control_unit::language_logic_unit::{
            boolean_eval_params::BooleanEvalParams,
            openai::{
                OpenAIClient,
                chat_completion_models::{
                    OpenAIChatCompletionRequest, OpenAIChatCompletionRequestText,
                },
                embeddings_models::OpenAIEmbeddingsRequest,
                model_config::{ModelEmbeddingsConfig, ModelTextConfig},
            },
            text_generation_config::TextGenerationConfig,
        },
        registers::ContextMessage,
    },
};

pub mod boolean_eval_params;
mod openai;
pub mod text_generation_config;

pub struct LanguageLogicUnit;

impl LanguageLogicUnit {
    fn default_text_model(model: &str, overrides: &TextModelOverrides) -> ModelTextConfig {
        ModelTextConfig {
            stream: overrides.stream.unwrap_or(false),
            return_progress: overrides.return_progress.unwrap_or(false),
            model: model.to_string(),
            reasoning_format: overrides
                .reasoning_format
                .clone()
                .unwrap_or_else(|| "auto".to_string()),
            temperature: overrides.temperature.unwrap_or(0.8),
            dynatemp_range: overrides.dynatemp_range.unwrap_or(0.0),
            dynatemp_exponent: overrides.dynatemp_exponent.unwrap_or(1.0),
            top_k: overrides.top_k.unwrap_or(40),
            top_p: overrides.top_p.unwrap_or(0.95),
            min_p: overrides.min_p.unwrap_or(0.05),
            xtc_probability: overrides.xtc_probability.unwrap_or(0.0),
            xtc_threshold: overrides.xtc_threshold.unwrap_or(0.1),
            typ_p: overrides.typ_p.unwrap_or(1.0),
            max_tokens: overrides.max_tokens.unwrap_or(-1),
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
            repeat_last_n: overrides.repeat_last_n.unwrap_or(64),
            repeat_penalty: overrides.repeat_penalty.unwrap_or(1.0),
            presence_penalty: overrides.presence_penalty.unwrap_or(0.0),
            frequency_penalty: overrides.frequency_penalty.unwrap_or(0.0),
            dry_multiplier: overrides.dry_multiplier.unwrap_or(0.0),
            dry_base: overrides.dry_base.unwrap_or(1.75),
            dry_allowed_length: overrides.dry_allowed_length.unwrap_or(2),
            dry_penalty_last_n: overrides.dry_penalty_last_n.unwrap_or(-1),
            timings_per_token: overrides.timings_per_token.unwrap_or(false),
        }
    }

    fn default_embeddings_model(model: &str) -> ModelEmbeddingsConfig {
        ModelEmbeddingsConfig {
            model: model.to_string(),
            encoding_format: "float".to_string(),
        }
    }

    fn trim_response(value: &str) -> String {
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
        let validation_err =
            |msg: String| Err(Exception::LanguageLogic(BaseException::new(msg, None)));

        if messages.len() < 2 {
            return validation_err(
                "Messages must contain at least a system and a user message.".to_string(),
            );
        }

        if messages[0].role != roles::SYSTEM_ROLE {
            return validation_err("The first message must be a system message.".to_string());
        }

        if messages[1].role != roles::USER_ROLE {
            return validation_err("The second message must be a user message.".to_string());
        }

        let mut expected_role = roles::ASSISTANT_ROLE;
        for message in messages.iter().skip(2) {
            if message.role != expected_role {
                return validation_err(format!(
                    "Unexpected role '{}' in messages, expected '{}'.",
                    message.role, expected_role
                ));
            }

            expected_role = if expected_role == roles::ASSISTANT_ROLE {
                roles::USER_ROLE
            } else {
                roles::ASSISTANT_ROLE
            };
        }

        if messages.last().map(|m| m.role.as_str()) != Some(roles::USER_ROLE) {
            return validation_err(format!(
                "Messages must end with a user message, but the last message has role '{}'.",
                messages
                    .last()
                    .map(|m| m.role.as_str())
                    .unwrap_or("unknown")
            ));
        }

        Ok(())
    }

    fn call_chat_api(
        content: &str,
        context: &[ContextMessage],
        text_generation_config: &TextGenerationConfig,
    ) -> Result<String, Exception> {
        let model = Self::default_text_model(
            &text_generation_config.text_model,
            &text_generation_config.text_model_overrides,
        );
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

        let messages = Self::merge_messages_by_role(&messages).map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by(
                "Failed to merge consecutive messages by role.",
                e,
            ))
        })?;
        Self::validate_messages(&messages).map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by(
                "Message sequence validation failed.",
                e,
            ))
        })?;

        if text_generation_config.debug_chat {
            println!("--- Chat Messages ---");
            for message in &messages {
                println!("Role: {}, Content: {}", message.role, message.content);
            }
            println!("---------------------");
        }

        let request = OpenAIChatCompletionRequest::new(messages, model);
        let response = OpenAIClient::chat_completion(
            &text_generation_config.base_url,
            &text_generation_config.chat_completion_endpoint,
            text_generation_config.timeout_secs,
            request,
        )
        .map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by(
                "OpenAI chat completion request failed.",
                e,
            ))
        })?;

        let choice = response.choices.first().ok_or_else(|| {
            Exception::LanguageLogic(BaseException::new(
                "No choices returned from chat completion.".to_string(),
                None,
            ))
        })?;

        Ok(Self::trim_response(&choice.message.content))
    }

    fn fetch_embeddings(
        content: &str,
        embedding_model: &str,
        base_url: &str,
        embeddings_endpoint: &str,
        timeout_secs: u64,
    ) -> Result<Vec<f32>, Exception> {
        let model = Self::default_embeddings_model(embedding_model);
        let request = OpenAIEmbeddingsRequest::new(content, model);
        let response =
            OpenAIClient::embeddings(base_url, embeddings_endpoint, timeout_secs, request)
                .map_err(|e| {
                    Exception::LanguageLogic(BaseException::caused_by(
                        "OpenAI embeddings request failed.",
                        e,
                    ))
                })?;

        let embedding = response.data.first().ok_or_else(|| {
            Exception::LanguageLogic(BaseException::new(
                "No embeddings returned from client.".to_string(),
                None,
            ))
        })?;

        Ok(embedding.embedding.to_owned())
    }

    pub fn cosine_similarity(
        value_a: &str,
        value_b: &str,
        embedding_model: &str,
        base_url: &str,
        embeddings_endpoint: &str,
        timeout_secs: u64,
    ) -> Result<u32, Exception> {
        let value_a_embeddings = Self::fetch_embeddings(
            value_a,
            embedding_model,
            base_url,
            embeddings_endpoint,
            timeout_secs,
        )
        .map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by("Failed to embed first value.", e))
        })?;
        let value_b_embeddings = Self::fetch_embeddings(
            value_b,
            embedding_model,
            base_url,
            embeddings_endpoint,
            timeout_secs,
        )
        .map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by("Failed to embed second value.", e))
        })?;

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

    pub fn generate_text(
        micro_prompt: &str,
        context: &[ContextMessage],
        text_generation_config: &TextGenerationConfig,
    ) -> Result<String, Exception> {
        Self::call_chat_api(micro_prompt, context, text_generation_config).map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by("Chat completion failed.", e))
        })
    }

    fn best_similarity_match(
        value: &str,
        candidates: &[String],
        embedding_model: &str,
        base_url: &str,
        embeddings_endpoint: &str,
        timeout_secs: u64,
    ) -> Result<u32, Exception> {
        candidates
            .iter()
            .map(|candidate| {
                Self::cosine_similarity(
                    &value.to_lowercase(),
                    &candidate.to_lowercase(),
                    embedding_model,
                    base_url,
                    embeddings_endpoint,
                    timeout_secs,
                )
                .map_err(|e| {
                    Exception::LanguageLogic(BaseException::caused_by(
                        "Failed to compute cosine similarity.",
                        e,
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|scores| scores.into_iter().max().unwrap_or(0))
    }

    pub fn evaluate_boolean(
        micro_prompt: &str,
        eval_params: &BooleanEvalParams,
        context: &[ContextMessage],
        text_generation_config: &TextGenerationConfig,
        embeddings_endpoint: &str,
    ) -> Result<u32, Exception> {
        let value =
            Self::generate_text(micro_prompt, context, text_generation_config).map_err(|e| {
                Exception::LanguageLogic(BaseException::caused_by(
                    "Text generation for boolean evaluation failed.",
                    e,
                ))
            })?;

        let max_true_score = Self::best_similarity_match(
            &value,
            &eval_params.true_values,
            &eval_params.embedding_model,
            &text_generation_config.base_url,
            embeddings_endpoint,
            text_generation_config.timeout_secs,
        )
        .map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by(
                "Failed to compute similarity score for true candidates.",
                e,
            ))
        })?;
        let max_false_score = Self::best_similarity_match(
            &value,
            &eval_params.false_values,
            &eval_params.embedding_model,
            &text_generation_config.base_url,
            embeddings_endpoint,
            text_generation_config.timeout_secs,
        )
        .map_err(|e| {
            Exception::LanguageLogic(BaseException::caused_by(
                "Failed to compute similarity score for false candidates.",
                e,
            ))
        })?;

        if max_true_score > max_false_score {
            Ok(100)
        } else {
            Ok(0)
        }
    }
}
