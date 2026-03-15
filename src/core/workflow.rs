use crate::core::ChannelDispatcher;
use crate::core::errors::WorkflowError;
use crate::core::messages::{DispatchOutgoingMessage, GenerateCompletion, HandleIncomingMessage};
use crate::core::model::{
    ChannelKind, ChatMessage, ConversationId, MessageId, OutgoingMessage, Role,
};
use crate::providers::StubProvider;
use actix::prelude::*;
use tracing::{debug, error, info};

// 1. Workflowアクターの構造体定義
pub struct Workflow {
    // 他のアクターへメッセージを送るためのアドレス（Addr）を保持します
    pub provider_addr: Addr<StubProvider>,
    pub channel_dispatcher_addr: Addr<ChannelDispatcher>,
}

impl Workflow {
    /// メッセージをChatMessageに変換し、履歴を組み立てる
    fn prepare_chat_history(&self, content: String) -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: Role::User,
            content,
        }]
    }

    /// Provider (LLM) に対して GenerateCompletion を送信し、結果を取得する
    async fn get_ai_completion(
        provider: Addr<StubProvider>,
        conversation_id: ConversationId,
        history: Vec<ChatMessage>,
    ) -> Result<String, WorkflowError> {
        let completion_req = GenerateCompletion {
            conversation_id,
            history,
            system_prompt: Some("あなたはZariganiという名前のAIアシスタントです。".to_string()),
        };

        match provider.send(completion_req).await {
            Ok(Ok(res)) => Ok(res.content),
            Ok(Err(e)) => Err(WorkflowError::Provider(e)),
            Err(e) => Err(WorkflowError::Mailbox(format!(
                "Failed to communicate with Provider: {:?}",
                e
            ))),
        }
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

        match dispatcher.send(reply_req).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(WorkflowError::Channel(e)),
            Err(e) => Err(WorkflowError::Mailbox(format!(
                "Failed to communicate with ChannelDispatcher: {:?}",
                e
            ))),
        }
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
        // メソッド抽出した処理を順次実行
        let provider = self.provider_addr.clone();
        let dispatcher = self.channel_dispatcher_addr.clone();
        let content = msg.message.content.clone();
        let history = self.prepare_chat_history(content.clone());
        let msg_conversation_id = msg.message.conversation_id;
        let msg_in_reply_to = msg.message.message_id;
        let msg_kind = msg.message.kind;
        let participant_id = msg.message.participant_id.0;
        let incoming_content = content;
        let msg_conversation_id_for_provider = msg_conversation_id.clone();

        Box::pin(async move {
            debug!(
                actor = "workflow",
                participant_id = %participant_id,
                conversation_id = %msg_conversation_id_for_provider.0,
                channel_kind = ?msg_kind,
                content = %incoming_content,
                "incoming message received"
            );

            // ステップ1: AIからの回答を取得
            let response_content = match Self::get_ai_completion(
                provider,
                msg_conversation_id_for_provider.clone(),
                history,
            )
            .await
            {
                Ok(content) => content,
                Err(err) => {
                    error!(actor = "workflow", error = %err, "failed to get AI completion");
                    return Err(err);
                }
            };

            // ステップ2: Channelへ回答を送信
            if let Err(err) = Self::dispatch_reply(
                dispatcher,
                msg_kind,
                msg_conversation_id.clone(),
                msg_in_reply_to,
                response_content,
            )
            .await
            {
                error!(
                    actor = "workflow",
                    conversation_id = %msg_conversation_id.0,
                    error = %err,
                    "failed to dispatch reply"
                );
                return Err(err);
            }

            info!(
                actor = "workflow",
                conversation_id = %msg_conversation_id.0,
                participant_id = %participant_id,
                "workflow successfully processed message"
            );
            Ok(())
        })
    }
}
