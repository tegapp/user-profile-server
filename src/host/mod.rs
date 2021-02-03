use chrono::prelude::*;
use async_graphql::{
    FieldResult,
    ID,
    Context,
};

pub mod resolvers;

use crate::machine::Machine;

#[derive(Debug, Clone)]
pub struct Host {
    pub id: crate::DbId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Props
    pub identity_public_key: String,
    pub slug: String,
    // pub server_version: String,
    // pub name: Option<String>,
}

#[async_graphql::Object]
impl Host {
    async fn id(&self) -> ID {
        self.id.into()
    }

    // async fn identity_public_key(&self) -> &String {
    //     &self.identity_public_key
    // }

    async fn slug(&self) -> &String {
        &self.slug
    }

    // async fn name(&self) -> &Option<String> {
    //     &self.name
    // }

    async fn machines<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> FieldResult<Vec<Machine>> {
        let db: &crate::Db = ctx.data()?;

        let hosts = sqlx::query_as!(
            Machine,
            r#"
                SELECT machines.* FROM machines
                WHERE machines.host_id=$1
            "#,
            self.id,
        )
            .fetch_all(db)
            .await?;

        Ok(hosts)
    }

}
