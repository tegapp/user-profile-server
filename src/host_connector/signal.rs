use async_graphql::ID;
use futures::SinkExt;
use std::{
    boxed::Box,
};
use eyre::{
    // eyre,
    Result,
    // Context as _,
};

use crate::ice_server::IceServer;

use super::HostConnector;

#[xactor::message(result = "Result<()>")]
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Signal {
    #[graphql(name = "userID")]
    pub user_id: async_graphql::ID,
    pub email: Option<String>,
    pub email_verified: bool,
    pub invite: Option<String>,
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub offer: async_graphql::Json<serde_json::Value>,
    pub ice_servers: Vec<IceServer>,
}

#[async_trait::async_trait]
impl xactor::Handler<Signal> for HostConnector {
    async fn handle(
        &mut self,
        _ctx: &mut xactor::Context<Self>,
        msg: Signal
    ) -> Result<()> {
        self.signals_sender.send(msg).await?;
        Ok(())
    }
}
