mod channels;
pub mod core;
pub mod providers;

use crate::channels::CliChannel;
use crate::core::{
    ChannelDispatcher, ChannelKind, ConversationId, HandleIncomingMessage, IncomingMessage,
    MessageId, ParticipantId, RegisterChannelRoute, Workflow,
};
use crate::providers::{OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig, StubProvider};
use actix::prelude::*;
use std::io::{self, BufRead, Write};
use tracing::{error, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[actix::main]
async fn main() {
    init_tracing();
    println!("Starting Zarigani system...");

    // 各アクターを起動
    let provider = build_provider_recipient();
    let channel_addr = CliChannel.start();
    let channel_dispatcher_addr = ChannelDispatcher::new().start();

    // ChannelDispatcherにCLIチャネルのルートを登録
    match channel_dispatcher_addr
        .send(RegisterChannelRoute {
            kind: ChannelKind::Cli,
            recipient: channel_addr.clone().recipient(),
        })
        .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            error!(actor = "main", error = %e, "failed to register CLI route");
            return;
        }
        Err(e) => {
            error!(
                actor = "main",
                target = "ChannelDispatcher",
                error = %e,
                "failed to communicate with ChannelDispatcher"
            );
            return;
        }
    }

    // WorkflowにAddrを渡して起動
    let workflow_addr = Workflow {
        provider,
        channel_dispatcher_addr,
    }
    .start();

    println!(
        "System initialized. Type your message and press Enter. Type /exit or Ctrl-D to quit."
    );

    run_cli_input_loop(workflow_addr).await;
}

fn build_provider_recipient() -> Recipient<crate::core::GenerateCompletion> {
    match OpenAiCompatibleProviderConfig::from_env() {
        Some(config) => match OpenAiCompatibleProvider::new(config) {
            Ok(provider) => provider.start().recipient(),
            Err(err) => {
                warn!(
                    actor = "main",
                    error = %err,
                    "failed to initialize OpenAI-compatible provider, falling back to stub provider"
                );
                StubProvider.start().recipient()
            }
        },
        None => {
            warn!(
                actor = "main",
                "OPENAI_BASE_URL and OPENAI_MODEL are not set, using stub provider"
            );
            StubProvider.start().recipient()
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("zarigani=debug,actix=info"));

    fmt().with_env_filter(filter).with_target(false).init();
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
                    Ok(Err(e)) => error!(actor = "main", error = %e, "failed to process message"),
                    Err(e) => warn!(
                        actor = "main",
                        target = "Workflow",
                        error = %e,
                        "failed to communicate with Workflow"
                    ),
                }
            }
            Err(e) => {
                error!(actor = "main", error = %e, "failed to read stdin");
                break;
            }
        }
    }
}
