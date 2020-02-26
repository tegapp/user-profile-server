use juniper::FieldResult;

use super::user::{ self, User };
use super::machine::{ self, Machine };
use crate::Context;

pub struct MyNamespace;

#[graphql_object(
    Context = Context,
)]
impl MyNamespace {
    async fn machines(context: &Context, slug: Option<String>) -> FieldResult<Vec<Machine>> {
        Ok(machine::my_machines(context, slug).await?)
    }
}

pub struct Query;

#[graphql_object(
    Context = Context,
)]
impl Query {
    // fn apiVersion() -> &str {
    //     "1.0"
    // }

    fn my() -> FieldResult<MyNamespace> {
        Ok(MyNamespace)
    }

    async fn current_user(context: &Context) -> FieldResult<User> {
        Ok(user::current_user(context).await?)
    }

    fn is_authenticated_for(context: &Context, machine_id: String) -> bool {
        context.user_id.is_some()
    }
}

pub struct Mutation;

#[graphql_object(
    Context = Context,
)]
impl Mutation {
    async fn login_with_password(
        context: &Context,
        input: user::LoginWithPasswordInput,
    ) -> FieldResult<User> {
        Ok(user::login_with_password(context, input).await?)
    }

    async fn signup(
        context: &Context,
        input: user::SignupInput,
    ) -> FieldResult<User> {
        Ok(user::signup(context, input).await?)
    }

    async fn create_machine(
        context: &Context,
        input: machine::CreateMachineInput
    ) -> FieldResult<Machine> {
        Ok(machine::create_machine(context, input).await?)
    }

    async fn set_machine_name(context: &Context, input: machine::SetMachineName) -> FieldResult<Machine> {
        Ok(machine::set_machine_name(context, input).await?)
    }

    async fn remove_machine(context: &Context, machine_id: String) -> FieldResult<Option<bool>> {
        Ok(machine::remove_machine(context, machine_id).await?)
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation)
}