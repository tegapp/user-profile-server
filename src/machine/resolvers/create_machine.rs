// use eyre::{
//     eyre,
//     Result,
//     Error,
//     // Context as _,
// };
// use super::Machine;

// #[derive(async_graphql::InputObject)]
// /// A new 3D printer or other CNC device
// pub struct CreateMachineInput {
//     pub public_key: String,
//     pub name: String,
//     pub slug: String,
// }

// pub async fn create_machine(context: &Context, input: CreateMachineInput) -> Result<Machine> {
//     let user_id = context.user_id().ok_or(unauthorized())?;

//     let machine = sqlx::query_as!(
//         Machine,
//         "
//             INSERT INTO machines (user_id, public_key, name, slug)
//             VALUES ($1, $2, $3, $4)
//             ON CONFLICT (user_id, slug)
//             DO UPDATE SET
//                 name=$3
//             RETURNING *
//         ",
//         user_id,
//         input.public_key,
//         input.name,
//         input.slug
//     )
//         .fetch_one(&mut context.sqlx_db().await?)
//         .await
//         .wrap_err( "Unable to insert new machine")?;

//     Ok(machine)
// }
