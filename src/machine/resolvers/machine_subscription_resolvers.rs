use async_graphql::*;
use futures::stream::{
    Stream,
    // StreamExt,
};

struct MachineSubscriptionResolvers;


#[derive(async_graphql::SimpleObject)]
pub struct Signal {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub offer: async_graphql::Json<serde_json::Value>,
}

#[Subscription]
impl MachineSubscriptionResolvers {
    async fn receive_signals<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = Signal>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        // TODO
        let stream = futures::stream::empty();
        Ok(stream)
    }
}
