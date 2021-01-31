use async_graphql::*;
use futures::stream::{
    Stream,
    // StreamExt,
};
use xactor::Actor;
use std::{
    pin::Pin,
    boxed::Box,
};

use crate::host_connector::{
    HostConnector,
    SignalsStream,
    StopHostConnector,
    Signal,
};

struct MachineSubscriptionResolvers;

#[Subscription]
impl MachineSubscriptionResolvers {
    async fn host_signals<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = Signal>> {
        let auth: &crate::AuthContext = ctx.data()?;
        let host_connectors: &crate::HostConnectorsMap = ctx.data()?;

        let host = auth.require_host()?;

        let (
            signals_sender,
            signals_receiver,
        ) = futures::channel::mpsc::unbounded();

        let next_host_connector = HostConnector {
            host_id: host.id,
            host_connectors: host_connectors.clone(),
            signals_sender,
        }.start().await?;

        let stream = SignalsStream {
            addr: next_host_connector.clone(),
            signals_receiver: Pin::new(Box::new(signals_receiver)),
        };

        // Drop the previous host connector if this is a duplicate
        let previous_host_connector = host_connectors
            .insert(host.id, next_host_connector.downgrade());

        if let Some(previous_host_connector) = previous_host_connector
            .and_then(|weak_addr| weak_addr.upgrade())
        {
            previous_host_connector.call(StopHostConnector).await?;
        }

        Ok(stream)
    }
}
