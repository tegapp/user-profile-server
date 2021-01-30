use futures_util::stream::TryStreamExt;

use crate::{ Context, ResultExt, unauthorized };
use super::Machine;

pub async fn my_machines(context: &Context, slug: Option<String>) -> crate::Result<Vec<Machine>> {
    let user_id = context.user_id().ok_or(unauthorized())?;

    let machines = if let Some(slug) = slug {
        sqlx::query_as!(
            Machine,
            "SELECT * FROM machines WHERE user_id=$1 AND slug=$2",
            user_id,
            slug
        )
            .fetch(&mut context.sqlx_db().await?)
            .try_collect()
            .await
            .wrap_err( "Unable to load my_machines from pg")?
    } else {
        sqlx::query_as!(
            Machine,
            "SELECT * FROM machines WHERE user_id=$1",
            user_id
        )
            .fetch(&mut context.sqlx_db().await?)
            .try_collect()
            .await
            .wrap_err( "Unable to load my_machines from pg")?
    };

    Ok(machines)
}
