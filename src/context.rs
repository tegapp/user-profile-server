use super::PgPool;
use juniper::{FieldResult, FieldError};
use super::PgPooledConnection;
use std::sync::Arc;
use crate::ResultExt;

pub struct Context {
    pub pool: PgPool,
    pub sqlx_pool: Arc<sqlx::PgPool>,
    pub user_id: Option<i32>,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

impl Context {
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