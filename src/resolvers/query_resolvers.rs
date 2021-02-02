// use eyre::{
//     // eyre,
//     Result,
//     Context as _,
// };
use async_graphql::{
    Context,
    // ID,
    FieldResult,
};

use crate::{ice_server::IceServer, user::User};

use super::my_namespace_resolvers::MyNamespace;

#[derive(Default, Clone, Copy)]
pub struct Query;

#[async_graphql::Object]
impl Query {
    async fn current_user<'ctx>(&self, ctx: &'ctx Context<'_>) -> FieldResult<Option<&'ctx User>> {
        let auth: &crate::AuthContext = ctx.data()?;

        Ok(auth.allow_unauthorized_user())
    }

    async fn my(&self) -> FieldResult<MyNamespace> {
        Ok(MyNamespace)
    }

    async fn ice_servers<'ctx>(&self, ctx: &'ctx Context<'_>) -> FieldResult<Vec<IceServer>> {
        let ice_servers: &crate::IceServerList = ctx.data()?;

        Ok((**ice_servers.load()).clone())
    }
}
