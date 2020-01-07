use super::schema::{users};

#[derive(Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub auth0_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
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