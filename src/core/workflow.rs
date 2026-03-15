use actix::prelude::*;
use crate::core::messages::{
    HandleIncomingMessage, ChatMessage, Role, GenerateCompletion, SendReply, WorkflowError
};
use crate::core::provider::Provider;
use crate::core::channel::Channel;

// 1. Workflowアクターの構造体定義
pub struct Workflow {
    // 他のアクターへメッセージを送るためのアドレス（Addr）を保持します
    pub provider_addr: Addr<Provider>,
    pub channel_addr: Addr<Channel>,
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
        provider: Addr<Provider>,
        history: Vec<ChatMessage>,
    ) -> Result<String, WorkflowError> {
        let completion_req = GenerateCompletion {
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

    /// Channel に対して SendReply を送信し、チャット投稿するよう指示する
    async fn dispatch_reply(
        channel: Addr<Channel>,
        target_channel_id: String,
        content: String,
    ) -> Result<(), WorkflowError> {
        let reply_req = SendReply {
            target_channel_id,
            content,
        };

        match channel.send(reply_req).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(WorkflowError::Channel(e)),
            Err(e) => Err(WorkflowError::Mailbox(format!(
                "Failed to communicate with Channel: {:?}",
                e
            ))),
        }
    }
}

// Actorトレイトの実装
impl Actor for Workflow {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Workflow actor started.");
    }
}

// 2. HandleIncomingMessage を受け取った際の処理（Handlerの実装）
impl Handler<HandleIncomingMessage> for Workflow {
    // 非同期処理 (.send().await) を行うため、戻り値を ResponseFuture にします
    type Result = ResponseFuture<Result<(), WorkflowError>>;

    fn handle(&mut self, msg: HandleIncomingMessage, _ctx: &mut Self::Context) -> Self::Result {
        // メソッド抽出した処理を順次実行
        let provider = self.provider_addr.clone();
        let channel = self.channel_addr.clone();
        let history = self.prepare_chat_history(msg.content.clone());

        Box::pin(async move {
            println!("Workflow received message from {}: {}", msg.user_id, msg.content);

            // ステップ1: AIからの回答を取得
            let response_content = Self::get_ai_completion(provider, history).await?;

            // ステップ2: Channelへ回答を送信
            Self::dispatch_reply(channel, msg.source_channel_id, response_content).await?;

            println!("Workflow successfully processed message.");
            Ok(())
        })
    }
}