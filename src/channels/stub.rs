use crate::core::errors::ChannelError;
use crate::core::messages::DispatchOutgoingMessage;
use actix::prelude::*;

pub struct StubChannel;

impl Actor for StubChannel {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Stub Channel actor started.");
    }
}

impl Handler<DispatchOutgoingMessage> for StubChannel {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: DispatchOutgoingMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("--------------------------------------------------");
        println!(
            "Stub Channel outgoing message for {:?} conversation {}: ",
            msg.message.kind, msg.message.conversation_id.0
        );
        println!("  >>> {}", msg.message.content);
        println!("--------------------------------------------------");
        Ok(())
    }
}
