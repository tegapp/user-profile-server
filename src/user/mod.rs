mod authorize_user;
pub use authorize_user::*;

pub mod jwt;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i32,
    pub firebase_uid: String,
    pub email: Option<String>,
    pub email_verified: bool,
}

#[graphql_object(
    description="A user"
)]
impl User {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn email(&self) -> &Option<String> {
        &self.email
    }
    fn email_verified(&self) -> bool {
        self.email_verified
    }
}
