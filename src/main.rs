#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;

pub mod schema;
pub mod user;
pub mod machine;
pub mod graphql_schema;
pub mod context;
// pub mod auth;

use warp::{http::Response, Filter};

use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager };
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

use context::Context;

// use auth::{ upsert_user };

error_chain::error_chain! {}

pub fn unauthorized() -> Error {
    "Unauthorized Access".into()
}

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
fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let log = warp::log("warp_server");

    let port = env::var("PORT")
        .expect("$PORT must be set")
        .parse()
        .expect("Invalid $PORT");

    let pool = establish_db_connection();

    let database_url = env::var("POSTGRESQL_ADDON_URI")
        .expect("$POSTGRESQL_ADDON_URI must be set");

    let sqlx_pool = futures::executor::block_on(async {
        sqlx::PgPool::new(&database_url)
            .await
            .map(|p| Arc::new(p))
            .expect("Could not connect to Postgres")
    });

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });


    let state = warp::any()
        .and(warp::header::optional::<String>("authorization"))
        // .and_then(move |authorization_header: Option<String>| {
        .map(move |authorization_header: Option<String>| {
            let pool = Arc::clone(&pool);
            let sqlx_pool = Arc::clone(&sqlx_pool);

            // upsert_user(
            //     Arc::clone(&pool),
            //     authorization_header,
            // )
            //     .map(move |result| {
            //         Context {
            //             pool: Arc::clone(&pool),
            //             sqlx_pool: Arc::clone(&sqlx_pool),
            //             user_id: result.ok().map(|user| user.id),
            //         }
            //     })
            //     .unit_error()
            //     .map_err(|_| warp::reject::not_found())
            //     .boxed()
            //     .compat()
            Context {
                pool: Arc::clone(&pool),
                sqlx_pool: Arc::clone(&sqlx_pool),
                user_id: None,
            }
        });

    let graphql_filter = juniper_warp::make_graphql_filter_async(
        crate::graphql_schema::schema(),
        state.boxed(),
    );

    let cors = warp::cors().allow_any_origin();

    warp::serve(
        warp::get2()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql"))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log)
            .with(cors),
    )
    .run(([0, 0, 0, 0], port));
}
