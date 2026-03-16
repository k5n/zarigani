use actix::prelude::*;
use rig::client::CompletionClient;
use rig::completion::{self, CompletionModel as _, Message};
use rig::providers::openai;
use std::env;
use tracing::{debug, error, info};

use crate::core::errors::ProviderError;
use crate::core::messages::{GenerateCompletion, ProviderResponse};
use crate::core::model::{ChatMessage, Role};

#[derive(Debug, Clone)]
pub struct OpenAiCompatibleProviderConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
}

impl OpenAiCompatibleProviderConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = env::var("OPENAI_BASE_URL").ok()?;
        let model = env::var("OPENAI_MODEL").ok()?;

        Some(Self {
            base_url,
            api_key: env::var("OPENAI_API_KEY").unwrap_or_else(|_| "dummy".to_string()),
            model,
            system_prompt: env::var("OPENAI_SYSTEM_PROMPT").ok(),
            temperature: env::var("OPENAI_TEMPERATURE")
                .ok()
                .and_then(|value| value.parse::<f64>().ok()),
            max_tokens: env::var("OPENAI_MAX_TOKENS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok()),
        })
    }
}

pub struct OpenAiCompatibleProvider {
    config: OpenAiCompatibleProviderConfig,
    client: openai::CompletionsClient,
}

impl OpenAiCompatibleProvider {
    pub fn new(config: OpenAiCompatibleProviderConfig) -> Result<Self, ProviderError> {
        let client = openai::Client::builder()
            .api_key(&config.api_key)
            .base_url(&config.base_url)
            .build()
            .map_err(|err| {
                ProviderError::General(format!("failed to build OpenAI-compatible client: {err}"))
            })?
            .completions_api();

        Ok(Self { config, client })
    }

    fn map_messages(history: Vec<ChatMessage>) -> Vec<Message> {
        history
            .into_iter()
            .filter_map(|message| match message.role {
                Role::System => None,
                Role::User => Some(Message::user(message.content)),
                Role::Assistant => Some(Message::assistant(message.content)),
            })
            .collect()
    }

    fn resolve_system_prompt(&self, request: &GenerateCompletion) -> Option<String> {
        request
            .system_prompt
            .clone()
            .or_else(|| self.config.system_prompt.clone())
    }

    fn split_prompt_and_history(
        mut history: Vec<Message>,
    ) -> Result<(Message, Vec<Message>), ProviderError> {
        let prompt = history.pop().ok_or_else(|| {
            ProviderError::General("provider request did not contain any chat messages".to_string())
        })?;

        Ok((prompt, history))
    }

    fn extract_text(
        response: completion::CompletionResponse<openai::completion::CompletionResponse>,
    ) -> Result<String, ProviderError> {
        let content = response
            .choice
            .iter()
            .filter_map(|item| match item {
                completion::AssistantContent::Text(text) => Some(text.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        if content.is_empty() {
            return Err(ProviderError::General(
                "provider response did not contain text content".to_string(),
            ));
        }

        Ok(content)
    }
}

impl Actor for OpenAiCompatibleProvider {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!(
            actor = "provider",
            provider = "openai-compatible",
            base_url = %self.config.base_url,
            model = %self.config.model,
            "actor started"
        );
    }
}

impl Handler<GenerateCompletion> for OpenAiCompatibleProvider {
    type Result = ResponseFuture<Result<ProviderResponse, ProviderError>>;

    fn handle(&mut self, msg: GenerateCompletion, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        let model_name = self.config.model.clone();
        let system_prompt = self.resolve_system_prompt(&msg);
        let mapped_history = Self::map_messages(msg.history);
        let temperature = self.config.temperature;
        let max_tokens = self.config.max_tokens;

        debug!(
            actor = "provider",
            provider = "openai-compatible",
            model = %model_name,
            history_len = mapped_history.len(),
            "completion request started"
        );

        Box::pin(async move {
            let (prompt, history) = Self::split_prompt_and_history(mapped_history)?;
            let model = client.completion_model(model_name.clone());
            let mut request = model.completion_request(prompt);

            if let Some(system_prompt) = system_prompt {
                request = request.preamble(system_prompt);
            }

            request = request.messages(history);
            request = request.temperature_opt(temperature);
            request = request.max_tokens_opt(max_tokens);

            let response = request.send().await.map_err(|err| {
                error!(
                    actor = "provider",
                    provider = "openai-compatible",
                    model = %model_name,
                    error = %err,
                    "completion request failed"
                );
                ProviderError::General(format!("provider request failed: {err}"))
            })?;

            let content = Self::extract_text(response)?;
            Ok(ProviderResponse { content })
        })
    }
}
