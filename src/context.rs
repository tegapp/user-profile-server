use std::sync::Arc;
use crate::ResultExt;
// use futures::prelude::*;

pub struct Context {
    pub sqlx_pool: Arc<sqlx::PgPool>,
    pub user: Option<crate::user::User>,
    // pub surf: surf::Client<http_client::native::NativeClient>,
    pub surf: Arc<surf::Client<http_client::native::NativeClient>>,
    pub auth_pem_keys: Arc<Vec<Vec<u8>>>,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

impl Context {
    pub async fn new(
        authorization_header: Option<String>,
        sqlx_pool: Arc<sqlx::PgPool>,
        surf_client: Arc<surf::Client<http_client::native::NativeClient>>,
        auth_pem_keys: Arc<Vec<Vec<u8>>>,
    ) -> Result<Self, crate::Error> {
        let mut context = Context {
            sqlx_pool,
            user: None,
            surf: surf_client,
            auth_pem_keys,
        };

        if let Some(authorization_header) = authorization_header {
            context.user = Some(crate::user::authorize_user(
                &context,
                authorization_header,
            ).await?);
        }

        Ok(context)
    }

    pub async fn sqlx_db(
        &self
    ) -> crate::Result<sqlx::pool::PoolConnection<sqlx::PgConnection>> {
        self.sqlx_pool
            .acquire()
            .await
            .chain_err(|| "Unable to acquire connection from sqlx pool")
    }

    pub fn user_id(&self) -> Option<i32> {
        self.user.as_ref().map(|user| user.id)
    }
}
