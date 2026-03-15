mod channels;
pub mod core;
pub mod providers;

use crate::channels::StubChannel;
use crate::core::{
    ChannelDispatcher, ChannelKind, ConversationId, HandleIncomingMessage, IncomingMessage,
    MessageId, ParticipantId, RegisterChannelRoute, Workflow,
};
use crate::providers::StubProvider;
use actix::prelude::*;

#[actix::main]
async fn main() {
    println!("Starting Zarigani system...");

    // 1. 各アクターを起動
    let provider_addr = StubProvider.start();
    let channel_addr = StubChannel.start();
    let channel_dispatcher_addr = ChannelDispatcher::new().start();

    match channel_dispatcher_addr
        .send(RegisterChannelRoute {
            kind: ChannelKind::Cli,
            recipient: channel_addr.clone().recipient(),
        })
        .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            eprintln!("Failed to register CLI route: {:?}", e);
            return;
        }
        Err(e) => {
            eprintln!("Failed to communicate with ChannelDispatcher: {:?}", e);
            return;
        }
    }

    // WorkflowにAddrを渡して起動
    let workflow_addr = Workflow {
        provider_addr,
        channel_dispatcher_addr,
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
