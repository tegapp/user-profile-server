use super::PgPool;
use juniper::{FieldResult, FieldError};
use super::PgPooledConnection;
use std::sync::Arc;
use crate::ResultExt;

pub struct Context {
    pub pool: PgPool,
    pub sqlx_pool: Arc<sqlx::PgPool>,
    pub user: Option<crate::user::User>,
    pub surf: surf::Client,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

impl Context {
    pub async fn new(
        authorization_header: String,
        pool: PgPool,
        sqlx_pool: Arc<sqlx::PgPool>,
        surf_client: Arc<surf::Client>,
    ) -> Result<User, Box<dyn Error>> {
        let mut context = Context {
            pool: Arc::clone(&pool),
            sqlx_pool: Arc::clone(&sqlx_pool),
            user: None,
            surf: Arc::clone(&surf_client),
        };

        context.user = crate::user::authorize_user(
            &context,
            authorization_header,
        ).await?;

        context
    }

    pub fn db(&self) -> FieldResult<PgPooledConnection> {
        self.pool
            .get()
            .map_err(|e| {
                FieldError::new(
                    format!("Could not open connection to the database {}", e.to_string()),
                    graphql_value!({ "internal_error": "Connection refused" })
                )
            })
    }

    pub async fn sqlx_db(
        &self
    ) -> crate::Result<sqlx::pool::PoolConnection<sqlx::PgConnection>> {
        self.sqlx_pool
            .acquire()
            .await
            .chain_err(|| "Unable to acquire connection from sqlx pool")
    }
}