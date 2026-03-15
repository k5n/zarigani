use crate::core::errors::ChannelError;
use crate::core::messages::DispatchOutgoingMessage;
use actix::prelude::*;

pub struct Channel;

impl Actor for Channel {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Mock Channel actor started.");
    }
}

impl Handler<DispatchOutgoingMessage> for Channel {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: DispatchOutgoingMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("--------------------------------------------------");
        println!(
            "Mock Channel outgoing message for {:?} conversation {}: ",
            msg.message.kind, msg.message.conversation_id.0
        );
        println!("  >>> {}", msg.message.content);
        println!("--------------------------------------------------");
        Ok(())
    }
}
