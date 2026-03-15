use std::io::{self, Write};

use crate::core::errors::ChannelError;
use crate::core::messages::DispatchOutgoingMessage;
use actix::prelude::*;

pub struct CliChannel;

impl Actor for CliChannel {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Cli Channel actor started.");
    }
}

impl Handler<DispatchOutgoingMessage> for CliChannel {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: DispatchOutgoingMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!();
        println!("zarigani> {}", msg.message.content);
        io::stdout().flush().ok();
        Ok(())
    }
}
