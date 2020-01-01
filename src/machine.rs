use super::schema::{machines};

#[derive(Identifiable, Queryable)]
// #[belongs_to(User)]
pub struct Machine {
    pub id: i32,
    pub user_id: i32,
    pub public_key: String,
    pub name: String,
    pub slug: String,
}

#[derive(Insertable)]
#[table_name="machines"]
pub struct NewMachine<'a> {
    pub user_id: i32,
    pub public_key: &'a str,
    pub name: &'a str,
    pub slug: &'a str,
}