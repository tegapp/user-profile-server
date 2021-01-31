use futures::stream::{
    Stream,
    // StreamExt,
};
use std::{
    boxed::Box,
    sync::Arc,
};

mod signal;
pub use signal::Signal;

mod host_connection;
pub use host_connection::{
    HostConnection,
    HostConnectionResponse,
};

pub struct HostConnector {
    pub host_id: crate::DbId,
    pub host_connectors: Arc<dashmap::DashMap<crate::DbId, xactor::WeakAddr<HostConnector>>>,
    pub signals_sender: futures::channel::mpsc::UnboundedSender<Signal>,
}

#[async_trait::async_trait]
impl xactor::Actor for HostConnector {
    async fn stopped(&mut self, ctx: &mut xactor::Context<Self>) {
        // Remove self from the host connectors map
        self.host_connectors.remove_if(&self.host_id, |_, addr| {
            addr.actor_id() == ctx.actor_id()
        });
    }
}

#[xactor::message(result = "()")]
pub struct StopHostConnector;

#[async_trait::async_trait]
impl xactor::Handler<StopHostConnector> for HostConnector {
    async fn handle(
        &mut self,
        ctx: &mut xactor::Context<Self>,
        _msg: StopHostConnector
    ) -> () {
        ctx.stop(None)
    }
}

pub struct SignalsStream {
    pub addr: xactor::Addr<HostConnector>,
    pub signals_receiver: std::pin::Pin<std::boxed::Box<futures::channel::mpsc::UnboundedReceiver<Signal>>>,
}

impl Stream for SignalsStream {
    type Item = Signal;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>
    ) -> core::task::Poll<Option<Self::Item>> {
        self.signals_receiver.as_mut().poll_next(cx)
    }
}
