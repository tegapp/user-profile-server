use super::schema::{users};

mod login_with_password;
pub use login_with_password::*;

mod signup;
pub use signup::*;

mod current_user;
pub use current_user::*;

#[derive(Identifiable, Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hashed_password: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
}

#[graphql_object(
    description="A user"
)]
impl User {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn username(&self) -> String {
        self.username.to_string()
    }
    fn email(&self) -> &Option<String> {
        &self.email
    }
    fn email_verified(&self) -> bool {
        self.email_verified
    }
}

#[derive(Insertable, AsChangeset)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub hashed_password: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
}