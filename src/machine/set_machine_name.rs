use crate::{
    Context,
    ResultExt,
    unauthorized,
};
use super::Machine;

#[graphql(description="set the cnc machine's name")]
#[derive(juniper::GraphQLInputObject)]
pub struct SetMachineName {
    pub id: String,
    pub name: String,
}

pub async fn set_machine_name(context: &Context, input: SetMachineName) -> crate::Result<Machine> {
    let user_id = context.user.ok_or(unauthorized())?.id;

    let machine = sqlx::query_as!(
        Machine,
        "
            UPDATE machines
            SET name = $3
            WHERE user_id=$1 AND id=$2
            RETURNING *
        ",
        user_id,
        input.id.parse::<i32>().chain_err(|| "Invalid machine id")?,
        input.name
    )
        .fetch_one(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to set machine name")?;

    Ok(machine)
}