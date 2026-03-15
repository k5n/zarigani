mod channels;
pub mod core;
pub mod providers;

use crate::channels::StubChannel;
use crate::core::{ChannelKind, ConversationId, IncomingMessage, MessageId, ParticipantId};
use crate::core::{HandleIncomingMessage, Workflow};
use crate::providers::StubProvider;
use actix::prelude::*;

#[actix::main]
async fn main() {
    println!("Starting Zarigani system...");

    // 1. 各アクターを起動
    let provider_addr = StubProvider.start();
    let channel_addr = StubChannel.start();

    // WorkflowにAddrを渡して起動
    let workflow_addr = Workflow {
        provider_addr,
        channel_addr: channel_addr.clone(),
    }
    .start();

    println!("System initialized. Testing message flow...");

    // 2. テスト用メッセージの送信 (CLIからメッセージが来た想定)
    let test_msg = HandleIncomingMessage {
        message: IncomingMessage {
            kind: ChannelKind::Cli,
            conversation_id: ConversationId("conv-cli-001".to_string()),
            participant_id: ParticipantId("test-user-456".to_string()),
            message_id: Some(MessageId("msg-001".to_string())),
            reply_to: None,
            content: "こんにちは！".to_string(),
        },
    };

    match workflow_addr.send(test_msg).await {
        Ok(Ok(_)) => println!("Main received: Workflow processed message successfully."),
        Ok(Err(e)) => eprintln!("Main received error from Workflow: {:?}", e),
        Err(e) => eprintln!("Main failed to communicate with Workflow: {:?}", e),
    }

    println!("Shutting down...");
}
