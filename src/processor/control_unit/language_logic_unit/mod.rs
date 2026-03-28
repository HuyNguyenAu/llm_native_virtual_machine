use crate::{
    assembler::roles,
    config::TextModelOverrides,
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
        text_model_overrides: &TextModelOverrides,
        debug_chat: bool,
    ) -> Result<String, Exception> {
        let model = Self::default_text_model(text_model, text_model_overrides);
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

        let messages = Self::merge_messages_by_role(&messages)?;
        Self::validate_messages(&messages)?;

        if debug_chat {
            println!("--- Chat Messages ---");
            for message in &messages {
                println!("Role: {}, Content: {}", message.role, message.content);
            }
            println!("---------------------");
        }

        let request = OpenAIChatCompletionRequest::new(messages, model);
        let response = OpenAIClient::chat_completion(request)?;

        let choice = response.choices.first().ok_or_else(|| {
            Exception::LanguageLogic(BaseException::new(
                "No choices returned from chat completion.".to_string(),
                None,
            ))
        })?;

        Ok(Self::clean_string(&choice.message.content))
    }

    fn embeddings(content: &str, embedding_model: &str) -> Result<Vec<f32>, Exception> {
        let model = Self::default_embeddings_model(embedding_model);
        let request = OpenAIEmbeddingsRequest::new(content, model);
        let response = OpenAIClient::embeddings(request)?;

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
    ) -> Result<u32, Exception> {
        let value_a_embeddings = Self::embeddings(value_a, embedding_model)?;
        let value_b_embeddings = Self::embeddings(value_b, embedding_model)?;

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
        text_model_overrides: &TextModelOverrides,
        debug_chat: bool,
    ) -> Result<String, Exception> {
        Self::chat(
            micro_prompt,
            context,
            text_model,
            text_model_overrides,
            debug_chat,
        )
    }

    pub fn boolean(
        micro_prompt: &str,
        true_values: &[&str],
        false_values: &[&str],
        context: &[ContextMessage],
        text_model: &str,
        embedding_model: &str,
        text_model_overrides: &TextModelOverrides,
        debug_chat: bool,
    ) -> Result<u32, Exception> {
        let value = Self::string(
            micro_prompt,
            context,
            text_model,
            text_model_overrides,
            debug_chat,
        )?;

        let max_true_score = true_values
            .iter()
            .map(|tv| {
                Self::cosine_similarity(&value.to_lowercase(), &tv.to_lowercase(), embedding_model)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(0);

        let max_false_score = false_values
            .iter()
            .map(|fv| {
                Self::cosine_similarity(&value.to_lowercase(), &fv.to_lowercase(), embedding_model)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(0);

        if max_true_score > max_false_score {
            Ok(100)
        } else {
            Ok(0)
        }
    }
}
