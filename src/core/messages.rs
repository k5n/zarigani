use actix::prelude::*;

use crate::core::errors::{ChannelError, ProviderError, WorkflowError};
use crate::core::model::{ChatMessage, ConversationId, IncomingMessage, OutgoingMessage};

#[derive(Message, Debug)]
#[rtype(result = "Result<(), WorkflowError>")]
pub struct HandleIncomingMessage {
    pub message: IncomingMessage,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<ProviderResponse, ProviderError>")]
pub struct GenerateCompletion {
    pub conversation_id: ConversationId,
    pub history: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<(), ChannelError>")]
pub struct DispatchOutgoingMessage {
    pub message: OutgoingMessage,
}
