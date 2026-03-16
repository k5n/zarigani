mod channels;
mod config;
pub mod core;
pub mod providers;

use crate::channels::CliChannel;
use crate::config::AppConfig;
use crate::core::{
    ChannelDispatcher, ChannelKind, ConversationId, HandleIncomingMessage, IncomingMessage,
    MessageId, ParticipantId, RegisterChannelRoute, Workflow,
};
use crate::providers::{OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig};
use actix::prelude::*;
use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};
use tracing::{error, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[actix::main]
async fn main() {
    init_tracing();
    if let Err(err) = run().await {
        error!(actor = "main", error = %err, "failed to start Zarigani system");
        std::process::exit(1);
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("zarigani=debug,actix=info"));

    fmt().with_env_filter(filter).with_target(false).init();
}

async fn run() -> Result<()> {
    println!("Starting Zarigani system...");
    let app_config = AppConfig::load_default().context("failed to load application config")?;

    // 各アクターを起動
    let provider = build_provider_recipient(app_config.provider.openai_compatible)
        .context("failed to initialize openai-compatible provider")?;
    let channel_addr = CliChannel.start();
    let channel_dispatcher_addr = ChannelDispatcher::new().start();

    // ChannelDispatcherにCLIチャネルのルートを登録
    let _ = channel_dispatcher_addr
        .send(RegisterChannelRoute {
            kind: ChannelKind::Cli,
            recipient: channel_addr.clone().recipient(),
        })
        .await
        .context("failed to communicate with ChannelDispatcher")?
        .map_err(|e| anyhow::anyhow!("failed to register CLI route: {e}"))?;

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
    Ok(())
}

fn build_provider_recipient(
    config: OpenAiCompatibleProviderConfig,
) -> Result<Recipient<crate::core::GenerateCompletion>> {
    let provider = OpenAiCompatibleProvider::new(config)
        .context("failed to build OpenAI-compatible provider")?;
    Ok(provider.start().recipient())
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
