use juniper::{FieldResult, FieldError};

use diesel::{
    QueryDsl,
    RunQueryDsl,
    ExpressionMethods,
};

use super::user::{ User, SignupInput };
use super::machine::{ Machine, CreateMachine, SetMachineName };
use super::context::Context;

fn unauthorized() -> FieldError {
    FieldError::new(
        "Unauthorized Access".to_string(),
        graphql_value!({ "internal_error": "Unauthorized" }),
    )
}

pub struct MyNamespace;

#[graphql_object(
    // Here we specify the context type for the object.
    // We need to do this in every type that
    // needs access to the context.
    Context = Context,
)]
impl MyNamespace {
    // Arguments to resolvers can either be simple types or input objects.
    // To gain access to the context, we specify a argument
    // that is a reference to the Context type.
    // Juniper automatically injects the correct context here.
    fn machines(context: &Context, slug: Option<String>) -> FieldResult<Vec<Machine>> {
        use super::schema::machines::dsl;

        let user_id = context.user_id.ok_or(unauthorized())?;

        // let results = machines::table.load::<Machine>(&context.db()?);
        let query = dsl::machines
            .filter(dsl::user_id.eq(user_id));

        if let Some(slug) = slug {
            let results = query
                .filter(dsl::slug.eq(slug))
                .get_results(&context.db()?)?;
            return Ok(results)
        } else {
            return Ok(query.get_results(&context.db()?)?);
        }
    }
}

pub struct Query;

#[graphql_object(
    // Here we specify the context type for the object.
    // We need to do this in every type that
    // needs access to the context.
    Context = Context,
)]
impl Query {
    // fn apiVersion() -> &str {
    //     "1.0"
    // }

    fn my() -> FieldResult<MyNamespace> {
        Ok(MyNamespace)
    }

    fn current_user(context: &Context) -> FieldResult<User> {
        use super::schema::users::dsl;

        let user_id = context.user_id.ok_or(unauthorized())?;

        let result = dsl::users
            .find(user_id)
            .get_result(&context.db()?)?;

        Ok(result)
    }

    fn is_authenticated_for(context: &Context, machine_id: String) -> bool {
        context.user_id.is_some()
    }
}

// Now, we do the same for our Mutation type.

pub struct Mutation;

#[graphql_object(
    Context = Context,
)]
impl Mutation {
    fn signup(
        context: &Context,
        input: SignupInput,
    ) -> FieldResult<Option<User>> {
        User::signup(context, input)
    }

    fn create_machine(context: &Context, input: CreateMachine) -> FieldResult<Machine> {
        use crate::diesel::RunQueryDsl;

        use super::schema::machines::{self, dsl};
        use super::machine::{ NewMachineSQL };

        let user_id = context.user_id.ok_or(unauthorized())?;

        let machine = NewMachineSQL {
            user_id,
            public_key: input.public_key,
            name: input.name,
            slug: input.slug,
        };

        let result = diesel::insert_into(machines::table)
            .values(&machine)
            .on_conflict((dsl::user_id, dsl::slug))
            .do_update()
            .set(&machine)
            .get_result(&context.db()?)?;

        Ok(result)
    }

    fn set_machine_name(context: &Context, input: SetMachineName) -> FieldResult<Machine> {
        use super::schema::machines::dsl;
        use super::machine::{ SetMachineNameSQL };

        let user_id = context.user_id.ok_or(unauthorized())?;

        let machine = SetMachineNameSQL {
            name: input.name,
        };

        let result = diesel::update(
            dsl::machines
                .filter(dsl::user_id.eq(user_id))
                .find(input.id.parse::<i32>()?)
        )
            .set(&machine)
            .get_result(&context.db()?)?;

        Ok(result)
    }

    fn remove_machine(context: &Context, machine_id: String) -> FieldResult<Option<bool>> {
        use super::schema::machines::dsl;

        let user_id = context.user_id.ok_or(unauthorized())?;

        diesel::delete(
            dsl::machines
                .filter(dsl::user_id.eq(user_id))
                .find(machine_id.parse::<i32>()?)
        )
            .execute(&context.db()?)?;

        Ok(None)
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation)
}