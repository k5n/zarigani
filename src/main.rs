pub mod core;

use actix::prelude::*;
use crate::core::{Provider, Channel, Workflow, HandleIncomingMessage};

#[actix::main]
async fn main() {
    println!("Starting Zarigani system...");

    // 1. 各アクターを起動
    let provider_addr = Provider.start();
    let channel_addr = Channel.start();
    
    // WorkflowにAddrを渡して起動
    let workflow_addr = Workflow {
        provider_addr,
        channel_addr: channel_addr.clone(),
    }.start();

    println!("System initialized. Testing message flow...");

    // 2. テスト用メッセージの送信 (Discordからメッセージが来た想定)
    let test_msg = HandleIncomingMessage {
        source_channel_id: "test-channel-123".to_string(),
        user_id: "test-user-456".to_string(),
        content: "こんにちは！".to_string(),
    };

    match workflow_addr.send(test_msg).await {
        Ok(Ok(_)) => println!("Main received: Workflow processed message successfully."),
        Ok(Err(e)) => eprintln!("Main received error from Workflow: {:?}", e),
        Err(e) => eprintln!("Main failed to communicate with Workflow: {:?}", e),
    }

    println!("Shutting down...");
}

