use actix::prelude::*;
use crate::core::messages::{SendReply, ChannelError};

pub struct Channel;

impl Actor for Channel {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Mock Channel actor started.");
    }
}

impl Handler<SendReply> for Channel {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: SendReply, _ctx: &mut Self::Context) -> Self::Result {
        println!("--------------------------------------------------");
        println!("Mock Channel outgoing message to {}: ", msg.target_channel_id);
        println!("  >>> {}", msg.content);
        println!("--------------------------------------------------");
        Ok(())
    }
}
