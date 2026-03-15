use std::collections::HashMap;

use actix::prelude::*;

use crate::core::errors::ChannelError;
use crate::core::messages::{
    DispatchOutgoingMessage, RegisterChannelRoute, UnregisterChannelRoute,
};
use crate::core::model::ChannelKind;
use tracing::{debug, error, info, warn};

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
        info!(actor = "channel_dispatcher", "actor started");
    }
}

impl Handler<RegisterChannelRoute> for ChannelDispatcher {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: RegisterChannelRoute, _ctx: &mut Self::Context) -> Self::Result {
        if self.routes_by_kind.contains_key(&msg.kind) {
            warn!(
                actor = "channel_dispatcher",
                channel_kind = ?msg.kind,
                "register route request for already registered channel kind"
            );
            return Err(ChannelError::AlreadyRegistered(format!(
                "route for {:?} is already registered",
                msg.kind
            )));
        }

        self.routes_by_kind.insert(msg.kind, msg.recipient);
        debug!(
            actor = "channel_dispatcher",
            channel_kind = ?msg.kind,
            "registered channel route"
        );
        Ok(())
    }
}

impl Handler<UnregisterChannelRoute> for ChannelDispatcher {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: UnregisterChannelRoute, _ctx: &mut Self::Context) -> Self::Result {
        self.routes_by_kind.remove(&msg.kind);
        debug!(
            actor = "channel_dispatcher",
            channel_kind = ?msg.kind,
            "unregistered channel route"
        );
        Ok(())
    }
}

impl Handler<DispatchOutgoingMessage> for ChannelDispatcher {
    type Result = ResponseFuture<Result<(), ChannelError>>;

    fn handle(&mut self, msg: DispatchOutgoingMessage, _ctx: &mut Self::Context) -> Self::Result {
        let kind = msg.message.kind;
        let conversation_id = msg.message.conversation_id.clone();
        let recipient = self.routes_by_kind.get(&kind).cloned();

        Box::pin(async move {
            let Some(recipient) = recipient else {
                warn!(
                    actor = "channel_dispatcher",
                    channel_kind = ?kind,
                    conversation_id = %conversation_id.0,
                    "no route found for channel kind"
                );
                return Err(ChannelError::NotFound(format!(
                    "no route registered for {:?}",
                    kind
                )));
            };

            match recipient.send(msg).await {
                Ok(result) => {
                    debug!(actor = "channel_dispatcher", conversation_id = %conversation_id.0, channel_kind = ?kind, "dispatched message to route");
                    result
                }
                Err(err) => {
                    error!(
                        actor = "channel_dispatcher",
                        conversation_id = %conversation_id.0,
                        channel_kind = ?kind,
                        error = %err,
                        "failed to communicate with route"
                    );
                    Err(ChannelError::General(format!(
                        "failed to communicate with route: {:?}",
                        err
                    )))
                }
            }
        })
    }
}
