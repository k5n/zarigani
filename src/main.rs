mod channels;
pub mod core;
pub mod providers;

use crate::channels::CliChannel;
use crate::core::{
    ChannelDispatcher, ChannelKind, ConversationId, HandleIncomingMessage, IncomingMessage,
    MessageId, ParticipantId, RegisterChannelRoute, Workflow,
};
use crate::providers::StubProvider;
use actix::prelude::*;
use std::io::{self, BufRead, Write};

#[actix::main]
async fn main() {
    println!("Starting Zarigani system...");

    // 1. 各アクターを起動
    let provider_addr = StubProvider.start();
    let channel_addr = CliChannel.start();
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

    println!(
        "System initialized. Type your message and press Enter. Type /exit or Ctrl-D to quit."
    );

    run_cli_input_loop(workflow_addr).await;
}

async fn run_cli_input_loop(workflow_addr: Addr<Workflow>) {
    let stdin = io::stdin();
    let mut stdin = io::BufReader::new(stdin.lock());
    let mut message_id = 1u64;
    loop {
        print!("user> ");
        std::io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!("\nEOF received, exiting...");
                break;
            }
            Ok(_) => {
                let content = line.trim_end().to_string();

                if content.is_empty() {
                    continue;
                }

                if content == "/exit" {
                    println!("Exiting...\n");
                    break;
                }

                let message = HandleIncomingMessage {
                    message: IncomingMessage {
                        kind: ChannelKind::Cli,
                        conversation_id: ConversationId("cli:default".to_string()),
                        participant_id: ParticipantId("cli:user".to_string()),
                        message_id: Some(MessageId(format!("cli-msg-{message_id}"))),
                        reply_to: None,
                        content,
                    },
                };
                message_id += 1;

                match workflow_addr.send(message).await {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => eprintln!("Failed to process message: {:?}", e),
                    Err(e) => eprintln!("Failed to communicate with Workflow: {:?}", e),
                }
            }
            Err(e) => {
                eprintln!("Failed to read stdin: {:?}", e);
                break;
            }
        }
    }
}
