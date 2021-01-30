mod create_machine;
pub use create_machine::*;

mod set_machine_name;
pub use set_machine_name::*;

mod remove_machine;
pub use remove_machine::*;

mod my_machines;
pub use my_machines::*;

pub struct Machine {
    pub id: crate::DbID,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub user_id: crate::DbID,
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
