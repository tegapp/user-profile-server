use super::schema::{machines};

#[derive(Identifiable, Queryable)]
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

#[graphql(description="a new 3D printer or other CNC device")]
#[derive(juniper::GraphQLInputObject)]
pub struct CreateMachine {
    pub public_key: String,
    pub name: String,
    pub slug: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name="machines"]
pub struct NewMachineSQL {
    pub user_id: i32,
    pub public_key: String,
    pub name: String,
    pub slug: String,
}

#[graphql(description="set the cnc machine's name")]
#[derive(juniper::GraphQLInputObject)]
pub struct SetMachineName {
    pub id: String,
    pub name: String,
}

#[derive(AsChangeset)]
#[table_name="machines"]
pub struct SetMachineNameSQL {
    pub name: String,
}