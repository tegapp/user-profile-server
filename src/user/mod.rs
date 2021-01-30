use chrono::prelude::*;
use async_graphql::{
    // FieldResult,
    ID,
};

mod authorize_user;
pub use authorize_user::*;

pub mod jwt;

#[derive(Debug, Clone)]
pub struct User {
    pub id: crate::DbId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub firebase_uid: String,
    pub email: String,
    pub email_verified: bool,
}

#[async_graphql::Object]
impl User {
    async fn id(&self) -> ID {
        self.id.into()
    }

    async fn email(&self) -> String {
        self.email.to_string()
    }

    async fn email_verified(&self) -> bool {
        self.email_verified
    }

    async fn picture(&self) -> Option<url::Url> {
        use gravatar::{ Gravatar, Default::Http404, Rating };

        let url = Gravatar::new(&self.email)
            .set_size(Some(150))
            .set_rating(Some(Rating::Pg))
            .set_default(Some(Http404))
            .image_url()
            .to_string();

        Some(url::Url::parse(&url).ok()?)
    }

}
