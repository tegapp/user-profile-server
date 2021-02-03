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
use crate::host::Host;

pub struct MyNamespace;

#[async_graphql::Object]
impl MyNamespace {
    async fn hosts<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        // slug: Option<String>,
    ) -> FieldResult<Vec<Host>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let user = auth.require_authorized_user()?;

        let hosts = sqlx::query_as!(
            Host,
            r#"
                SELECT hosts.* FROM hosts
                INNER JOIN hosts_users ON hosts_users.host_id = hosts.id
                WHERE hosts_users.user_id=$1
            "#,
            user.id,
        )
            .fetch_all(db)
            .await
            .wrap_err( "Unable to load my.hosts")?;

        Ok(hosts)
    }
}
