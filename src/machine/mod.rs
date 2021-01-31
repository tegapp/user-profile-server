use chrono::prelude::*;
use async_graphql::{
    // FieldResult,
    ID,
};

mod resolvers;

pub struct Machine {
    pub id: crate::DbId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub user_id: crate::DbId,
    pub public_key: String,
    pub name: String,
    pub slug: String,
}

#[async_graphql::Object]
/// A 3D printer or other CNC device
impl Machine {
    async fn id(&self) -> ID {
        self.id.into()
    }
    async fn public_key(&self) -> &String {
        &self.public_key
    }
    async fn name(&self) -> &String {
        &self.name
    }
    async fn slug(&self) -> &String {
        &self.slug
    }
}
