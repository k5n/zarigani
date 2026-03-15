#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Discord,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConversationId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticipantId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingMessage {
    pub kind: ChannelKind,
    pub conversation_id: ConversationId,
    pub participant_id: ParticipantId,
    pub message_id: Option<MessageId>,
    pub reply_to: Option<MessageId>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMessage {
    pub kind: ChannelKind,
    pub conversation_id: ConversationId,
    pub content: String,
    pub in_reply_to: Option<MessageId>,
}
