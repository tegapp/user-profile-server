use super::schema::{machines};

mod create_machine;
pub use create_machine::*;

mod set_machine_name;
pub use set_machine_name::*;

mod remove_machine;
pub use remove_machine::*;

mod my_machines;
pub use my_machines::*;

// #[belongs_to(User)]
pub struct Machine {
    pub id: i32,
    pub user_id: i32,
    pub public_key: String,
    pub name: String,
    pub slug: String,
}

#[graphql_object(
    description="A user's 3D printer or other CNC device"
)]
impl Machine {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn public_key(&self) -> &String {
        &self.public_key
    }
    fn name(&self) -> &String {
        &self.name
    }
    fn slug(&self) -> &String {
        &self.slug
    }
}
