#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;

pub mod schema;
pub mod user;
pub mod machine;
pub mod graphql_schema;
pub mod context;
pub mod auth;

use futures::future::{FutureExt, TryFutureExt};

use warp::{http::Response, Filter};

use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager };
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

use context::Context;

use auth::{ upsert_user };


error_chain::error_chain! {
    // // The type defined for this error. These are the conventional
    // // and recommended names, but they can be arbitrarily chosen.
    // //
    // // It is also possible to leave this section out entirely, or
    // // leave it empty, and these names will be used automatically.
    // types {
    //     Error, ErrorKind, ResultExt, Result;
    // }

    // // Without the `Result` wrapper:
    // //
    // // types {
    // //     Error, ErrorKind, ResultExt;
    // // }

    // // Automatic conversions between this error chain and other
    // // error chains. In this case, it will e.g. generate an
    // // `ErrorKind` variant called `Another` which in turn contains
    // // the `other_error::ErrorKind`, with conversions from
    // // `other_error::Error`.
    // //
    // // Optionally, some attributes can be added to a variant.
    // //
    // // This section can be empty.
    // links {
    //     Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
    // }

    // // Automatic conversions between this error chain and other
    // // error types not defined by the `error_chain!`. These will be
    // // wrapped in a new error with, in the first case, the
    // // `ErrorKind::Fmt` variant. The description and cause will
    // // forward to the description and cause of the original error.
    // //
    // // Optionally, some attributes can be added to a variant.
    // //
    // // This section can be empty.
    // foreign_links {
    //     Fmt(::std::fmt::Error);
    //     Io(::std::io::Error) #[cfg(unix)];
    // }

    // // Define additional `ErrorKind` variants.  Define custom responses with the
    // // `description` and `display` calls.
    // errors {
    //     InvalidToolchainName(t: String) {
    //         description("invalid toolchain name")
    //         display("invalid toolchain name: '{}'", t)
    //     }

    //     // You can also add commas after description/display.
    //     // This may work better with some editor auto-indentation modes:
    //     UnknownToolchainVersion(v: String) {
    //         description("unknown toolchain version"), // note the ,
    //         display("unknown toolchain version: '{}'", v), // trailing comma is allowed
    //     }
    // }
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

    futures::executor::block_on(async {
        let users = sqlx::query_as!(
            user::User,
            "SELECT * FROM users ORDER BY id",
        )
            .fetch_all(&mut sqlx_pool.acquire().await.unwrap())
            .await.unwrap();

        println!("!!!!IT WORKED!!!!! {:?}", users);
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
        .and_then(move |authorization_header: Option<String>| {
            let pool = Arc::clone(&pool);

            upsert_user(
                Arc::clone(&pool),
                authorization_header,
            )
                .map(move |result| {
                    Context {
                        pool: Arc::clone(&pool),
                        user_id: result.ok().map(|user| user.id),
                    }
                })
                .unit_error()
                .map_err(|_| warp::reject::not_found())
                .boxed()
                .compat()
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
    .run(([127, 0, 0, 1], port));
}
