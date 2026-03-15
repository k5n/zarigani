use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider error: {0}")]
    General(String),

    #[error("Capacity exceeded")]
    CapacityExceeded,

    #[error("Authentication failed")]
    AuthError,
}

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Channel error: {0}")]
    General(String),

    #[error("Target not found: {0}")]
    NotFound(String),
}
