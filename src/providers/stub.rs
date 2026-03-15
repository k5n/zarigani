use crate::core::errors::ProviderError;
use crate::core::messages::{GenerateCompletion, ProviderResponse};
use actix::prelude::*;

pub struct StubProvider;

impl Actor for StubProvider {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Mock StubProvider actor started.");
    }
}

impl Handler<GenerateCompletion> for StubProvider {
    type Result = Result<ProviderResponse, ProviderError>;

    fn handle(&mut self, msg: GenerateCompletion, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "Mock Provider received request with {} history messages.",
            msg.history.len()
        );

        // Use the last user message as the response (echo logic)
        let last_content = msg
            .history
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "No input provided".to_string());

        Ok(ProviderResponse {
            content: format!("[Echo from Provider] {}", last_content),
        })
    }
}
