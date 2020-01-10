#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
extern crate dotenv;
extern crate reqwest;
extern crate futures;
extern crate futures03;
extern crate serde;
extern crate serde_json;
extern crate url;


// use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager };
use dotenv::dotenv;
use std::env;
use std::sync::Arc;


pub mod schema;
pub mod user;
pub mod machine;
pub mod hyper_server;
pub mod graphql_schema;
pub mod context;
pub mod auth;

pub type PgPool = Arc<Pool<ConnectionManager<PgConnection>>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_db_connection() -> PgPool {
    dotenv().ok();

    let database_url = env::var("POSTGRESQL_ADDON_URI")
        .expect("$POSTGRESQL_ADDON_URI must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url.clone());

    let pool = Pool::builder()
        .max_size(2)
        .build(manager)
        .expect(&format!("Error connecting to {}", database_url));

    Arc::new(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let connection = establish_db_connection();
    hyper_server::run(connection).await?;

    Ok(())
}
