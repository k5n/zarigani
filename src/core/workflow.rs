use crate::core::ChannelDispatcher;
use crate::core::errors::WorkflowError;
use crate::core::messages::{DispatchOutgoingMessage, GenerateCompletion, HandleIncomingMessage};
use crate::core::model::{
    ChannelKind, ChatMessage, ConversationId, IncomingMessage, MessageId, OutgoingMessage, Role,
};
use actix::prelude::*;
use tracing::{Instrument, debug, error, info, info_span};

// 1. Workflowアクターの構造体定義
pub struct Workflow {
    // 他のアクターへメッセージを送るためのアドレス（Addr）を保持します
    pub provider: Recipient<GenerateCompletion>,
    pub channel_dispatcher_addr: Addr<ChannelDispatcher>,
}

impl Workflow {
    /// メッセージをChatMessageに変換し、履歴を組み立てる
    fn prepare_chat_history(&self, content: &str) -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: Role::User,
            content: content.to_string(),
        }]
    }

    /// Provider (LLM) に対して GenerateCompletion を送信し、結果を取得する
    async fn get_ai_completion(
        provider: Recipient<GenerateCompletion>,
        conversation_id: ConversationId,
        history: Vec<ChatMessage>,
    ) -> Result<String, WorkflowError> {
        let completion_req = GenerateCompletion {
            conversation_id,
            history,
            system_prompt: Some("あなたはZariganiという名前のAIアシスタントです。".to_string()),
        };

        let res = provider
            .send(completion_req)
            .await
            .map_err(|e| {
                WorkflowError::Mailbox(format!("Failed to communicate with Provider: {:?}", e))
            })?
            .map_err(WorkflowError::Provider)?;

        Ok(res.content)
    }

    /// Channel へ送信するアクター内部メッセージを組み立てる
    async fn dispatch_reply(
        dispatcher: Addr<ChannelDispatcher>,
        kind: ChannelKind,
        conversation_id: ConversationId,
        in_reply_to: Option<MessageId>,
        content: String,
    ) -> Result<(), WorkflowError> {
        let reply_req = DispatchOutgoingMessage {
            message: OutgoingMessage {
                kind,
                conversation_id,
                content,
                in_reply_to,
            },
        };

        dispatcher
            .send(reply_req)
            .await
            .map_err(|e| {
                WorkflowError::Mailbox(format!(
                    "Failed to communicate with ChannelDispatcher: {:?}",
                    e
                ))
            })?
            .map_err(WorkflowError::Channel)
    }
}

// Actorトレイトの実装
impl Actor for Workflow {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!(actor = "workflow", "actor started");
    }
}

// 2. HandleIncomingMessage を受け取った際の処理（Handlerの実装）
impl Handler<HandleIncomingMessage> for Workflow {
    // 非同期処理 (.send().await) を行うため、戻り値を ResponseFuture にします
    type Result = ResponseFuture<Result<(), WorkflowError>>;

    fn handle(&mut self, msg: HandleIncomingMessage, _ctx: &mut Self::Context) -> Self::Result {
        let provider = self.provider.clone();
        let dispatcher = self.channel_dispatcher_addr.clone();

        let HandleIncomingMessage { message } = msg;
        let IncomingMessage {
            kind: msg_kind,
            conversation_id: msg_conversation_id,
            participant_id,
            message_id: msg_in_reply_to,
            content,
            ..
        } = message;

        let history = self.prepare_chat_history(&content);

        let span = info_span!(
            "handle_incoming_message",
            actor = "workflow",
            conversation_id = %msg_conversation_id.0,
            message_id = ?msg_in_reply_to,
            channel_kind = ?msg_kind,
            participant_id = %participant_id.0,
        );

        Box::pin(
            async move {
                debug!(content = %content, "incoming message received");
                debug!("provider completion request started");

                // ステップ1: AIからの回答を取得
                let response_content =
                    Self::get_ai_completion(provider, msg_conversation_id.clone(), history)
                        .await
                        .inspect_err(|err| {
                            error!(error = %err, "failed to get AI completion");
                        })?;

                // ステップ2: Channelへ回答を送信
                Self::dispatch_reply(
                    dispatcher,
                    msg_kind,
                    msg_conversation_id,
                    msg_in_reply_to,
                    response_content,
                )
                .await
                .inspect_err(|err| {
                    error!(error = %err, "failed to dispatch reply");
                })?;

                info!("workflow successfully processed message");
                Ok(())
            }
            .instrument(span),
        )
    }
}
