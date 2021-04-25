// use eyre::{
//     eyre,
//     // Result,
//     Context as _,
// };
use async_graphql::{Context, FieldResult, ID};
use dashmap::DashMap;
use std::{boxed::Box, hash::Hash, sync::Arc, time::Duration};
use futures::channel::oneshot;

use crate::host::Host;

pub struct HostConnection {
    pub host: Host,
    pub session_id: ID,
    pub add_to_host_users: bool,
}

#[derive(async_graphql::SimpleObject)]
pub struct HostConnectionResponse {
    pub answer: async_graphql::Json<serde_json::Value>,
    pub ice_candidates: Vec<async_graphql::Json<serde_json::Value>>,
}

struct DashMapDeleteOnDrop<K, V>
where
    K: Eq + Hash
{
    dash_map: Arc<DashMap<K, V>>,
    key: K,
}

impl<K, V> Drop for DashMapDeleteOnDrop<K, V>
where
    K: Eq + Hash
{
    fn drop(&mut self) {
        // trace!("Drop: {:?}", self.key);
        let _ = self.dash_map.remove(&self.key);
    }
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

        let drop_session = DashMapDeleteOnDrop {
            dash_map: Arc::clone(response_senders),
            key: key.clone(),
        };

        let _ = response_senders.insert(key, sender);

        let response = tokio::time::timeout(
            Duration::from_secs(30),
            receiver,
        ).await??;

        if self.add_to_host_users {
            if let Some(user) = auth.allow_unauthorized_user() {
                sqlx::query!(
                    r#"
                        INSERT INTO host_users (user_id, host_id, authorized_by_host)
                        VALUES ($1, $2, True)
                        ON CONFLICT (user_id, host_id)
                        DO
                            UPDATE SET authorized_by_host = TRUE
                        RETURNING id
                    "#,
                    user.id,
                    self.host.id,
                )
                    .fetch_one(db)
                    .await?;
            }
        };

        drop(drop_session);

        Ok(response)
    }
}
