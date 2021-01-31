use eyre::{
    // eyre,
    // Result,
    Context as _,
};
use async_graphql::{
    Context,
    ID,
    FieldResult,
};

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    // async fn create_machine(
    //     context: &Context,
    //     input: machine::CreateMachineInput
    // ) -> FieldResult<Machine> {
    //     Ok(machine::create_machine(context, input).await?)
    // }

    // async fn set_machine_name(context: &Context, input: machine::SetMachineName) -> FieldResult<Machine> {
    //     Ok(machine::set_machine_name(context, input).await?)
    // }

    async fn remove_user_from_machine<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        machine_id: ID
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let user = auth.require_authorized_user()?;

        let machine_id = machine_id.parse::<i64>().wrap_err( "Invalid machine id")?;

        sqlx::query!(
            "
                DELETE FROM machines WHERE user_id = $1 AND id = $2
            ",
            user.id,
            machine_id,
        )
            .execute(db)
            .await
            .wrap_err( "Unable to delete machine")?;

        Ok(None)
    }
}
