use crate::{ Context, ResultExt };
use super::User;

#[derive(juniper::GraphQLInputObject)]
pub struct SignupInput {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub async fn signup(
    context: &Context,
    input: SignupInput,
) -> crate::Result<User> {

    let SignupInput {
        username,
        password,
        email
    } = input;

    let hashed_password = {
        use bcrypt::{DEFAULT_COST, hash};

        hash(password, DEFAULT_COST)
            .chain_err(|| "unable to hash new password in sign up")?
    };


    let user = sqlx::query_as!(
        User,
        "
            INSERT INTO users (username, hashed_password, email)
            VALUES ($1, $2, $3)
            RETURNING *
        ",
        username,
        hashed_password,
        email
    )
        .fetch_one(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to save user in sign up")?;

    Ok(user)
}
