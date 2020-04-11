mod authorize_user;
pub use authorize_user::*;

pub mod jwt;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i32,
    pub firebase_uid: String,
    pub email: String,
    pub email_verified: bool,
}

#[graphql_object(
    description="A user"
)]
impl User {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn email(&self) -> String {
        self.email.to_string()
    }
    fn email_verified(&self) -> bool {
        self.email_verified
    }

    fn picture(&self) -> Option<url::Url> {
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
