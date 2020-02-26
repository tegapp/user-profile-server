use super::schema::{users};
use juniper::{FieldResult};
use crate::context::Context;

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

impl User {
    fn signup(
        context: &Context,
        input: SignupInput,
    ) -> FieldResult<Option<User>> {

        let SignupInput {
            username,
            password,
            email
        } = input;

        let hashed_password = {
            use bcrypt::{DEFAULT_COST, hash};

            hash(password, DEFAULT_COST)?
        };


    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct SignupInput<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str,
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