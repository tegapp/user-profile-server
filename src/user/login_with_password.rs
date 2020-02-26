use crate::{ Context, ResultExt };
use super::User;

#[derive(juniper::GraphQLInputObject)]
pub struct LoginWithPasswordInput {
    pub username: String,
    pub password: String,
}

pub async fn login_with_password(
    context: &Context,
    input: LoginWithPasswordInput,
) -> crate::Result<User> {
    const BAD_AUTH_ERR: &str = "Incorrect username or password";

    let LoginWithPasswordInput {
        username,
        password,
    } = input;

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username=$1",
        username
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to get user from pg in login")?
        .ok_or(BAD_AUTH_ERR)?;

    let hashed_password = user.hashed_password
        .as_ref()
        .ok_or(BAD_AUTH_ERR)?;

    let is_correct_password = bcrypt::verify(password, &hashed_password)
        .chain_err(|| BAD_AUTH_ERR)?;

    if !is_correct_password {
        Err(BAD_AUTH_ERR.into())
    } else {
        Ok(user)
    }
}
