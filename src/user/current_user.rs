use crate::{ Context, ResultExt, unauthorized };
use super::User;

pub async fn current_user(context: &Context) -> crate::Result<User> {
    let user_id = context.user_id.ok_or(unauthorized())?;

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id=$1",
        user_id
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to get current user from pg")?
        .ok_or(unauthorized())?;

    Ok(user)
}