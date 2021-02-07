// use eyre::{
//     eyre,
//     // Result,
//     Context as _,
// };
use async_graphql::{Context, FieldResult, ID};
use std::{boxed::Box, time::Duration};
use futures::channel::oneshot;

use crate::host::Host;

pub struct HostConnection {
    pub host: Host,
    pub session_id: ID,
}

#[derive(async_graphql::SimpleObject)]
pub struct HostConnectionResponse {
    pub answer: async_graphql::Json<serde_json::Value>,
    pub ice_candidates: Vec<async_graphql::Json<serde_json::Value>>,
}

#[async_graphql::Object]
impl HostConnection {
    async fn host(&self) -> &Host {
        &self.host
    }

    async fn response<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> FieldResult<HostConnectionResponse> {
        let db: &crate::Db = ctx.data()?;
        let response_senders: &crate::ConnectionResponseSenders = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let key = (self.host.id, self.session_id.clone());

        let (
            sender,
            receiver
        ) = oneshot::channel();

        let _ = response_senders.insert(key, sender);

        let response = tokio::time::timeout(
            Duration::from_secs(30),
            receiver,
        ).await??;

        // if the host authenticated the request then add the host to the users' host list
        if let Some(user) = auth.allow_unauthorized_user() {
            sqlx::query!(
                r#"
                    INSERT INTO hosts_users (user_id, host_id)
                    VALUES ($1, $2)
                    ON CONFLICT (user_id, host_id)
                    DO NOTHING
                    RETURNING *
                "#,
                user.id,
                self.host.id,
            )
                .fetch_one(db)
                .await?;
        };


        Ok(response)
    }
}
