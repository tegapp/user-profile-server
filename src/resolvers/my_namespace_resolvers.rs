use eyre::{
    // eyre,
    // Result,
    Context as _,
};
use async_graphql::{
    Context,
    // ID,
    FieldResult,
};
use crate::machine::Machine;

pub struct MyNamespace;

#[async_graphql::Object]
impl MyNamespace {
    async fn machines<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        // slug: Option<String>,
    ) -> FieldResult<Vec<Machine>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let user = auth.require_authorized_user()?;

        // let machines = if let Some(slug) = slug {
        //     sqlx::query_as!(
        //         Machine,
        //         r#"
        //             SELECT * FROM machines
        //             INNER JOIN hosts_users ON hosts_users.host_id = machines.host_id
        //             WHERE hosts_users.user_id=$1 AND machines.slug=$2
        //         "#,
        //         user.id,
        //         slug,
        //     )
        //         .fetch_all(db)
        //         .await
        //         .wrap_err( "Unable to load my.machines")?
        // } else {
        let machines = sqlx::query_as!(
            Machine,
            r#"
                SELECT machines.* FROM machines
                INNER JOIN hosts_users ON hosts_users.host_id = machines.host_id
                WHERE hosts_users.user_id=$1
            "#,
            user.id,
        )
            .fetch_all(db)
            .await
            .wrap_err( "Unable to load my.machines")?;
        // };

        Ok(machines)
    }
}
