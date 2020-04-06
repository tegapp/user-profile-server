#[macro_use] extern crate juniper;
#[macro_use] extern crate log;

pub mod user;
pub mod machine;
pub mod ice_server;
pub mod graphql_schema;
pub mod context;

use warp::{http::Response, Filter};

use std::env;
use std::sync::Arc;

use futures::prelude::*;

use context::Context;

use juniper::http::{ GraphQLRequest };

// use auth::{ upsert_user };

error_chain::error_chain! {}

pub fn unauthorized() -> Error {
    "Unauthorized Access".into()
}

#[derive(Debug)]
pub struct InternalServerError;

impl warp::reject::Reject for InternalServerError {}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init_timed();

    let log = warp::log("warp_server");

    let port = env::var("PORT")
        .expect("$PORT must be set")
        .parse()
        .expect("Invalid $PORT");

    let surf_client = Arc::new(surf::Client::new());

    let database_url = env::var("POSTGRESQL_ADDON_URI")
        .expect("$POSTGRESQL_ADDON_URI must be set");

    let sqlx_pool = futures::executor::block_on(async {
        sqlx::PgPool::new(&database_url)
            .await
            .map(|p| Arc::new(p))
            .expect("Could not connect to Postgres")
    });

    // Pre-caching ice servers and pem keys
    let ice_servers = Arc::new(ice_server::get_ice_servers().await?);
    let pem_keys = Arc::new(user::jwt::get_pem_keys().await?);

    tokio::spawn(async {
        info!("Firebase Certs and WebRTC ICE servers will refresh in an hour");
        tokio::time::delay_for(std::time::Duration::from_secs(60 * 60)).await;

        info!("Restarting server to refresh Firebase certs and WebRTC ICE servers");
        std::process::exit(0);
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
            Context::new(
                authorization_header.clone(),
                Arc::clone(&sqlx_pool),
                Arc::clone(&surf_client),
                Arc::clone(&pem_keys),
                Arc::clone(&ice_servers),
            )
                .map_err(|err| {
                    log::error!("Context Error: {:?}", err);

                    warp::reject::custom(crate::InternalServerError)
                })
        });

    async fn handle_graphql(
        req: GraphQLRequest,
        context: Context
    ) -> crate::Result<Response<Vec<u8>>> {
        use warp::http::{ StatusCode };

        let schema = crate::graphql_schema::schema();

        let graphql_res = req.execute(&schema, &context).await;

        let status_code = if graphql_res.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };

        let body = serde_json::to_vec(&graphql_res)
            .chain_err(|| "Unable to serialize graphql response")?;

        let res = Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(body)
            .chain_err(|| "Unable to build graphql response")?;

        Ok(res)
    }

    use warp::{http::Method};
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE])
        .allow_headers(vec!["authorization", "content-type"]);

    let cors_route = warp::options().map(warp::reply);

    let graphql_filter = warp::post()
        .and(warp::path("graphql"))
        .and(warp::body::content_length_limit(1024 * 1024))
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

    warp::serve(
        warp::any()
        //     // .and(warp::path("graphiql"))
        //     // .and(juniper_warp::graphiql_filter("/graphql"))
        //     // .or(homepage)
        //     // .and(homepage)
            .and(cors_route)
        //     // cors_route
            .or(graphql_filter)
            .or(homepage)
            .with(cors)
            .with(log),
    )
    .run(([0, 0, 0, 0], port))
    .await;

    Ok(())
}
