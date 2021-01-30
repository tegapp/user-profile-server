#[macro_use] extern crate tracing;

use async_graphql::{EmptySubscription, http::{playground_source, GraphQLPlaygroundConfig}};
use async_graphql::Schema;
use async_graphql_warp::{Response, graphql_subscription_with_data};
use ice_server::IceServer;
use sqlx::postgres::PgPoolOptions;
use user::jwt::PemKey;
use std::{sync::Arc};
use warp::{Filter, http::Response as HttpResponse, hyper::Method};
use eyre::{
    eyre,
    Result,
    Error,
    // Context as _,
};
use arc_swap::ArcSwap;

mod auth_context;
pub use auth_context::AuthContext;

pub mod user;
// pub mod machine;
pub mod ice_server;
// pub mod graphql_schema;
// pub mod context;


type Db = sqlx::Pool<sqlx::Postgres>;
type DbId = i64;

type PemKeyList = Arc<ArcSwap<Vec<PemKey>>>;
// type IceServerList = Arc<ArcSwap<Vec<IceServer>>>;
// type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn unauthorized() -> Error {
    eyre!("Unauthorized Access")
}

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    pub async fn name(&self) -> bool {
        false
    }
}

struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    pub async fn name(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct InternalServerError;
impl warp::reject::Reject for InternalServerError {}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT")
        .expect("$PORT must be set")
        .parse()
        .expect("Invalid $PORT");

    // let surf_client = Arc::new(surf::Client::new());

    let database_url = std::env::var("POSTGRESQL_ADDON_URI")
        .expect("$POSTGRESQL_ADDON_URI must be set");

    let db = PgPoolOptions::new()
        .connect(&database_url).await?;

    // Database migrations
    sqlx::migrate::Migrator::new(
        std::path::Path::new("./migrations")
    )
        .await?
        .run(&db)
        .await?;

    // Pre-caching ice servers and pem keys
    let ice_servers = ice_server::get_ice_servers().await?;
    let ice_servers = Arc::new(ArcSwap::from(Arc::new(
        ice_servers,
    )));

    let pem_keys = user::jwt::get_pem_keys().await?;
    let pem_keys = Arc::new(ArcSwap::from(Arc::new(
        pem_keys,
    )));


    let schema = Schema::build(
        QueryRoot,
        MutationRoot,
        EmptySubscription,
    )
        .data(db.clone())
        // .data(surf_client)
        .data(ice_servers.clone())
        .data(pem_keys.clone())
        .finish();


    tokio::spawn({
        let ice_servers = ice_servers.clone();
        let pem_keys = pem_keys.clone();

        async move {
            loop {
                info!("Firebase Certs and WebRTC ICE servers will refresh in an hour");
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;

                info!("Refreshing Firebase certs and WebRTC ICE servers...");

                // let pem_keys_for_refresh = Arc::clone(&pem_keys_for_refresh);
                // let ice_servers_for_refresh = Arc::clone(&ice_servers_for_refresh);

                // Pem Keys Refresh
                let next_pem_keys = user::jwt::get_pem_keys()
                    .await
                    .expect("Unable to refresh Firebase certs");

                pem_keys.store(Arc::new(next_pem_keys));

                // ICE Servers Refresh
                let next_ice_servers = ice_server::get_ice_servers().await
                    .expect("Unable to refresh Twilio ICE Servers");

                ice_servers.store(Arc::new(next_ice_servers));
            }
        }
    });

    info!("Playground: http://localhost:{}", port);

    let db_clone = db.clone();
    let pem_clone = pem_keys.clone();
    let graphql_post = async_graphql_warp::graphql(schema.clone())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::content_length_limit(1024 * 1024))
        .and_then(move |
            graphql_tuple,
            authorization_header,
        | {
            let db = db_clone.clone();
            let pem_keys = pem_clone.clone();

            async move {
                let (schema, request): (
                    Schema<QueryRoot, MutationRoot, EmptySubscription>,
                    async_graphql::Request,
                ) = graphql_tuple;

                let auth = AuthContext::http_post_auth(
                    &db,
                    &pem_keys,
                    authorization_header,
                ).await;

                let auth = match auth {
                    Ok(auth) => auth,
                    Err(err) => {
                        warn!("Auth Error: {:?}", err);
                        return Err(warp::reject::custom(crate::InternalServerError))
                    }
                };

                let request = request.data(auth);

                Ok(Response::from(schema.execute(request).await))
            }
        });

    let db_clone = db.clone();
    let graphql_subscription = graphql_subscription_with_data(
        schema,
        move |json| {
            let db = db_clone.clone();

            async move {
                let auth = AuthContext::websocket_auth(&db, json).await?;

                let mut data = async_graphql::Data::default();
                data.insert(auth);

                Ok(data)
            }
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(
                GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
            ))
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE])
        .allow_headers(vec!["authorization", "content-type"]);

    let cors_route = warp::options()
        .map(warp::reply);

    let routes = graphql_subscription
        .or(graphql_playground)
        .or(graphql_post)
        .or(cors_route)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
