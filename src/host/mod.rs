use chrono::prelude::*;
use async_graphql::{
    // FieldResult,
    ID,
};

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

    async fn identity_public_key(&self) -> &String {
        &self.identity_public_key
    }

    // async fn name(&self) -> &Option<String> {
    //     &self.name
    // }
}
