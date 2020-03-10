use crate::{ Context, ResultExt, unauthorized };
use super::Machine;

#[graphql(description="a new 3D printer or other CNC device")]
#[derive(juniper::GraphQLInputObject)]
pub struct CreateMachineInput {
    pub public_key: String,
    pub name: String,
    pub slug: String,
}

pub async fn create_machine(context: &Context, input: CreateMachineInput) -> crate::Result<Machine> {
    let user_id = context.user.ok_or(unauthorized())?.id;

    let machine = sqlx::query_as!(
        Machine,
        "
            INSERT INTO machines (user_id, public_key, name, slug)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, slug)
            DO UPDATE SET
                name=$3
            RETURNING *
        ",
        user_id,
        input.public_key,
        input.name,
        input.slug
    )
        .fetch_one(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to insert new machine")?;

    Ok(machine)
}