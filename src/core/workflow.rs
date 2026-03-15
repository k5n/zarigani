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
        // 非同期ブロック(async move)内で使うために、アドレスをクローンしておきます
        let provider = self.provider_addr.clone();
        let channel = self.channel_addr.clone();

        Box::pin(async move {
            println!("Workflow received message from {}: {}", msg.user_id, msg.content);

            // ステップ1: メッセージをChatMessageに変換し、履歴を組み立てる
            let chat_msg = ChatMessage {
                role: Role::User,
                content: msg.content.clone(),
            };

            // ステップ2: Provider (LLM) に対して GenerateCompletion を送信
            let completion_req = GenerateCompletion {
                history: vec![chat_msg], // フェーズ1では過去の履歴なしで、今回の発言のみ送る
                system_prompt: Some("あなたはZariganiという名前のAIアシスタントです。".to_string()),
            };

            // Providerからの返答を待機
            let provider_res = provider.send(completion_req).await;

            // 返答内容の取り出し（エラーハンドリングも含む）
            let response_content = match provider_res {
                Ok(Ok(res)) => res.content,
                Ok(Err(e)) => format!("[Provider Error] {:?}", e),
                Err(e) => format!("[Actix Mailbox Error] Failed to communicate with Provider: {:?}", e),
            };

            // ステップ3: Channel に対して SendReply を送信し、チャット投稿するよう指示
            let reply_req = SendReply {
                target_channel_id: msg.source_channel_id,
                content: response_content,
            };

            // Channelからの返答を待機（送信成功・失敗の確認）
            match channel.send(reply_req).await {
                Ok(Ok(_)) => {
                    println!("Workflow successfully forwarded reply to Channel.");
                    Ok(())
                }
                Ok(Err(e)) => {
                    eprintln!("[Channel Error] {:?}", e);
                    Err(WorkflowError::General("Channel failed to send message".to_string()))
                }
                Err(e) => {
                    eprintln!("[Actix Mailbox Error] Failed to communicate with Channel: {:?}", e);
                    Err(WorkflowError::General("Mailbox error with Channel".to_string()))
                }
            }
        })
    }
}