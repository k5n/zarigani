use std::collections::HashMap;

use actix::prelude::*;

use crate::core::errors::ChannelError;
use crate::core::messages::{
    DispatchOutgoingMessage, RegisterChannelRoute, UnregisterChannelRoute,
};
use crate::core::model::ChannelKind;
use tracing::{Instrument, debug, error, info, info_span, warn};

pub struct ChannelDispatcher {
    routes_by_kind: HashMap<ChannelKind, Recipient<DispatchOutgoingMessage>>,
}

impl ChannelDispatcher {
    pub fn new() -> Self {
        Self {
            routes_by_kind: HashMap::new(),
        }
    }

    fn register_route(
        &mut self,
        kind: ChannelKind,
        recipient: Recipient<DispatchOutgoingMessage>,
    ) -> Result<(), ChannelError> {
        if self.routes_by_kind.contains_key(&kind) {
            warn!(
                actor = "channel_dispatcher",
                channel_kind = ?kind,
                "register route request for already registered channel kind"
            );
            return Err(ChannelError::AlreadyRegistered(format!(
                "route for {:?} is already registered",
                kind
            )));
        }

        self.routes_by_kind.insert(kind, recipient);
        debug!(
            actor = "channel_dispatcher",
            channel_kind = ?kind,
            "registered channel route"
        );
        Ok(())
    }

    fn unregister_route(&mut self, kind: ChannelKind) {
        self.routes_by_kind.remove(&kind);
        debug!(
            actor = "channel_dispatcher",
            channel_kind = ?kind,
            "unregistered channel route"
        );
    }

    async fn dispatch_message(
        recipient: Recipient<DispatchOutgoingMessage>,
        msg: DispatchOutgoingMessage,
    ) -> Result<(), ChannelError> {
        recipient.send(msg).await.map_err(|err| {
            error!(error = %err, "failed to communicate with route");
            ChannelError::General(format!("failed to communicate with route: {:?}", err))
        })?
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
        self.register_route(msg.kind, msg.recipient)
    }
}

impl Handler<UnregisterChannelRoute> for ChannelDispatcher {
    type Result = Result<(), ChannelError>;

    fn handle(&mut self, msg: UnregisterChannelRoute, _ctx: &mut Self::Context) -> Self::Result {
        self.unregister_route(msg.kind);
        Ok(())
    }
}

impl Handler<DispatchOutgoingMessage> for ChannelDispatcher {
    type Result = ResponseFuture<Result<(), ChannelError>>;

    fn handle(&mut self, msg: DispatchOutgoingMessage, _ctx: &mut Self::Context) -> Self::Result {
        let kind = msg.message.kind;
        let conversation_id = msg.message.conversation_id.clone();
        let recipient = self.routes_by_kind.get(&kind).cloned();

        let span = info_span!(
            "dispatch_outgoing_message",
            actor = "channel_dispatcher",
            channel_kind = ?kind,
            conversation_id = %conversation_id.0,
        );

        Box::pin(
            async move {
                let recipient = recipient.ok_or_else(|| {
                    warn!("no route found for channel kind");
                    ChannelError::NotFound(format!("no route registered for {:?}", kind))
                })?;
                Self::dispatch_message(recipient, msg).await?;
                debug!("dispatched message to route");
                Ok(())
            }
            .instrument(span),
        )
    }
}
