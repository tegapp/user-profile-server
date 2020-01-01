use super::schema::{users};

#[derive(Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub auth0_id: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub auth0_id: &'a str,
}