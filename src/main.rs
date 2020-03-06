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
pub mod warp_sessions;

use warp::{http::Response, Filter};

use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager };
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

use context::Context;

use juniper::http::{ GraphQLRequest };

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

    use crate::warp_sessions::{ Session, SQLXStore };

    let store = SQLXStore {
        secret: "TODO: SESSION SECRET".to_string(),
        pool: Arc::clone(&sqlx_pool),
    };

    let state = warp::any()
        // .or(
        //     warp::header::<String>("authorization")
        //         .map(move |authorization_header| {
        //             // Bearer Auth
        //             None
        //         })
        // )
        // .unify()
        .and(
            warp_sessions::optional_csrf_session(store)
        )
        .map(move |session: Session, csrf_token: Option<String>| -> Context {
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
                session,
                csrf_token,
            }
        });

    // let graphql_filter = juniper_warp::make_graphql_filter_async(
    //     crate::graphql_schema::schema(),
    //     state.boxed(),
    // );

    use futures::prelude;

    async fn handle_graphql(
        req: GraphQLRequest,
        context: Context
    ) -> crate::Result<Response<Vec<u8>>> {
        use crate::graphql_schema::schema;
        use warp::http::{ self, StatusCode };

        let graphql_res = req.execute_async(&schema(), &context).await;

        let status_code = if graphql_res.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };

        let session_cookie = context.session.cookie_builder()
            // TODO: production domain/secure toggles
            .domain("localhost")
            .secure(false)
            .finish()
            .to_string();

        let body = serde_json::to_vec(&graphql_res)
            .chain_err(|| "Unable to serialize graphql response")?;

        let res = Response::builder()
            .status(status_code)
            .header(
                http::header::SET_COOKIE,
                session_cookie,
            )
            .body(body)
            .chain_err(|| "Unable to build graphql response")?;

        Ok(res)
    }


    #[derive(Debug)]
    struct InternalServerError;

    impl warp::reject::Reject for InternalServerError {}

    let graphql_filter = warp::post()
        .and(warp::path("graphql"))
        .and(warp::body::json())
        .and(state)
        .and_then(|req, context| async move {
            handle_graphql(req, context)
                .await
                .map_err(|err| {
                    log::error!("GraphQL Error: {:?}", err);

                    warp::reject::custom(InternalServerError)
                })
        });

    let cors = warp::cors().allow_any_origin();

    warp::serve(
        warp::get()
            // .and(warp::path("graphiql"))
            // .and(juniper_warp::graphiql_filter("/graphql"))
            // .or(homepage)
            .and(homepage)
            .or(graphql_filter)
            .with(log)
            .with(cors),
    )
    .run(([0, 0, 0, 0], port));
}
