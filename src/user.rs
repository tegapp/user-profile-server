use super::schema::{users};

#[derive(Identifiable, Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub auth0_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
}

#[juniper::object(
    description="A user"
)]
impl User {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn name(&self) -> &Option<String> {
        &self.name
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
    pub auth0_id: &'a str,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
}