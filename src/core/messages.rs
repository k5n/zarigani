use actix::prelude::*;
use thiserror::Error;

/// AIとの会話における役割
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// 抽象化された1つのメッセージ単位
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

// --- Workflow ---

/// Discordなどの発信元からWorkflowへ送られるイベント
#[derive(Message, Debug)]
#[rtype(result = "Result<(), WorkflowError>")]
pub struct HandleIncomingMessage {
    pub source_channel_id: String,
    pub user_id: String,
    pub content: String,
}

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Workflow error: {0}")]
    General(String),

    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Channel error: {0}")]
    Channel(#[from] ChannelError),

    #[error("Mailbox error: {0}")]
    Mailbox(String),
}

// --- Provider ---

/// WorkflowからLLM(Provider)への生成依頼
#[derive(Message, Debug)]
#[rtype(result = "Result<ProviderResponse, ProviderError>")]
pub struct GenerateCompletion {
    pub history: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
}

/// LLMからの返答の抽象化
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider error: {0}")]
    General(String),
    #[error("Capacity exceeded")]
    CapacityExceeded,
    #[error("Authentication failed")]
    AuthError,
}

// --- Channel ---

/// WorkflowからChannelへの送信指示
#[derive(Message, Debug)]
#[rtype(result = "Result<(), ChannelError>")]
pub struct SendReply {
    pub target_channel_id: String,
    pub content: String,
}

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Channel error: {0}")]
    General(String),
    #[error("Target not found: {0}")]
    NotFound(String),
}
