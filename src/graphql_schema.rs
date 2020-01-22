use juniper::{FieldResult, FieldError};

use super::machine::{ Machine, CreateMachine, SetMachineName };
use super::context::Context;

// #[derive(juniper::GraphQLEnum)]
// enum Episode {
//     NewHope,
//     Empire,
//     Jedi,
// }

pub struct MyNamespace;

#[juniper::object(
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
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        
        use super::schema::machines::dsl;

        // let results = machines::table.load::<Machine>(&context.db()?);
        let query = dsl::machines
            .filter(dsl::user_id.eq(context.user_id));

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

#[juniper::object(
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

    fn is_authenticated_for(machine_id: String) -> bool {
        true
    }
}

// Now, we do the same for our Mutation type.

pub struct Mutation;

#[juniper::object(
    Context = Context,
)]
impl Mutation {
    fn create_machine(context: &Context, input: CreateMachine) -> FieldResult<Machine> {
        use crate::diesel::RunQueryDsl;
        
        use super::schema::machines;
        use super::machine::{ NewMachineSQL };

        let machine = NewMachineSQL {
            user_id: context.user_id,
            public_key: input.public_key,
            name: input.name,
            slug: input.slug,
        };

        let result = diesel::insert_into(machines::table)
            .values(&machine)
            .get_result(&context.db()?)
            .expect("Error saving new post");

        Ok(result)
    }

    fn set_machine_name(context: &Context, input: SetMachineName) -> FieldResult<Machine> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        
        use super::schema::machines::dsl;
        use super::machine::{ SetMachineNameSQL };

        let machine = SetMachineNameSQL {
            name: input.name,
        };

        let result = diesel::update(
            dsl::machines
                .filter(dsl::user_id.eq(context.user_id))
                .find(input.id.parse::<i32>()?)
        )
            .set(&machine)
            .get_result(&context.db()?)?;

        Ok(result)
    }

    fn remove_machine(context: &Context, machine_id: String) -> FieldResult<Option<bool>> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        
        use super::schema::machines::dsl;

        diesel::delete(
            dsl::machines
                .filter(dsl::user_id.eq(context.user_id))
                .find(machine_id.parse::<i32>()?)
        )
            .execute(&context.db()?)?;

        Ok(None)
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, Mutation>;
