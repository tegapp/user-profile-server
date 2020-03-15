use crate::{ Context, ResultExt, unauthorized };

pub async fn remove_machine(context: &Context, machine_id: String) -> crate::Result<Option<bool>> {
    let user_id = context.user_id().ok_or(unauthorized())?;

    sqlx::query!(
        "
            DELETE FROM machines WHERE user_id = $1 AND id = $2
        ",
        user_id,
        machine_id.parse::<i32>().chain_err(|| "Invalid machine id")?
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to delete machine")?;

    Ok(None)
}