use std::collections::HashMap;

use actix::prelude::*;

use crate::core::errors::ChannelError;
use crate::core::messages::{
    DispatchOutgoingMessage, RegisterChannelRoute, UnregisterChannelRoute,
};
use crate::core::model::ChannelKind;

pub struct ChannelDispatcher {
    routes_by_kind: HashMap<ChannelKind, Recipient<DispatchOutgoingMessage>>,
}

impl ChannelDispatcher {
    pub fn new() -> Self {
        Self {
            routes_by_kind: HashMap::new(),
        }
    }
}

impl Default for ChannelDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for ChannelDispatcher {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("ChannelDispatcher actor started.");
    }
}

impl Handler<RegisterChannelRoute> for ChannelDispatcher {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: RegisterChannelRoute, _ctx: &mut Self::Context) -> Self::Result {
        if self.routes_by_kind.contains_key(&msg.kind) {
            return Err(ChannelError::AlreadyRegistered(format!(
                "route for {:?} is already registered",
                msg.kind
            )));
        }

        self.routes_by_kind.insert(msg.kind, msg.recipient);
        Ok(())
    }
}

impl Handler<UnregisterChannelRoute> for ChannelDispatcher {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: UnregisterChannelRoute, _ctx: &mut Self::Context) -> Self::Result {
        self.routes_by_kind.remove(&msg.kind);
        Ok(())
    }
}

impl Handler<DispatchOutgoingMessage> for ChannelDispatcher {
    type Result = ResponseFuture<Result<(), ChannelError>>;

    fn handle(&mut self, msg: DispatchOutgoingMessage, _ctx: &mut Self::Context) -> Self::Result {
        let recipient = self.routes_by_kind.get(&msg.message.kind).cloned();

        Box::pin(async move {
            let Some(recipient) = recipient else {
                return Err(ChannelError::NotFound(format!(
                    "no route registered for {:?}",
                    msg.message.kind
                )));
            };

            match recipient.send(msg).await {
                Ok(result) => result,
                Err(err) => Err(ChannelError::General(format!(
                    "failed to communicate with route: {:?}",
                    err
                ))),
            }
        })
    }
}
