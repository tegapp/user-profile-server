#[macro_use] extern crate tracing;
#[macro_use] extern crate nanoid;
// #[macro_use] extern crate lazy_static;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;
use async_graphql_warp::{graphql_subscription_with_data};
use dashmap::DashMap;
use futures::channel::oneshot;
use host_connector::{HostConnectionResponse, HostConnector};
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

mod b58_fingerprint;
pub use b58_fingerprint::b58_fingerprint;

pub mod host;
pub mod host_connector;
pub mod ice_server;
pub mod machine;
pub mod protos;
pub mod resolvers;
pub mod user;

type Db = sqlx::Pool<sqlx::Postgres>;
type DbId = i64;

type PemKeyList = Arc<ArcSwap<Vec<PemKey>>>;
type IceServerList = Arc<ArcSwap<Vec<IceServer>>>;
type HostConnectorsMap = Arc<DashMap<crate::DbId, xactor::WeakAddr<HostConnector>>>;
type ConnectionResponseSenders = DashMap<
    (crate::DbId, async_graphql::ID),
    oneshot::Sender<HostConnectionResponse>,
>;

pub fn unauthorized() -> Error {
    eyre!("Unauthorized Access")
}

pub struct Void;

#[async_graphql::Object]
impl Void {
    pub async fn id(&self) -> async_graphql::ID {
        "VOID".into()
    }
}

// Schema definition
type AppSchema = Schema<Query, Mutation, Subscription>;

#[derive(async_graphql::MergedObject, Default, Clone, Copy)]
pub struct Query(
    resolvers::query_resolvers::Query,
);

#[derive(async_graphql::MergedObject, Default, Clone, Copy)]
pub struct Mutation(
    host::resolvers::host_mutation_resolvers::HostMutation,
);

// #[derive(async_graphql::MergedSubscription, Default, Clone, Copy)]
// pub struct Subscription(
//     host::resolvers::host_subscription_resolvers::HostSubscription,
// );

pub type Subscription = host::resolvers::host_subscription_resolvers::HostSubscription;


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
        .or(std::env::var("DATABASE_URL"))
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

    let host_connectors: HostConnectorsMap = Arc::new(DashMap::new());
    let connection_response_senders: ConnectionResponseSenders = DashMap::new();

    let schema = Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
        .extension(async_graphql::extensions::Tracing::default())
        // .extension(async_graphql::extensions::Logger)
        .data(db.clone())
        // .data(surf_client)
        .data(ice_servers.clone())
        .data(pem_keys.clone())
        .data(host_connectors)
        .data(connection_response_senders)
        .finish();

    tokio::spawn({
        let ice_servers = ice_servers.clone();
        let pem_keys = pem_keys.clone();

        async move {
            loop {
                info!("Firebase Certs and WebRTC ICE servers will refresh in an hour");
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;

                info!("Refreshing Firebase certs and WebRTC ICE servers...");

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
        .and(warp::header::optional::<String>("x-host-identity-public-key"))
        .and(warp::body::content_length_limit(1024 * 1024))
        .and_then(move |
            graphql_tuple,
            authorization_header,
            host_identity_public_key,
        | {
            let db = db_clone.clone();
            let pem_keys = pem_clone.clone();

            let (schema, request): (
                AppSchema,
                async_graphql::Request,
            ) = graphql_tuple;

            AuthContext::http_post_auth(
                db,
                pem_keys,
                authorization_header,
                host_identity_public_key,
                schema,
                request,
            )
        });

    let db_clone = db.clone();
    let graphql_subscription = graphql_subscription_with_data(
        schema,
        move |json| {
            let db = db_clone.clone();

            AuthContext::websocket_auth(db, json)
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

    let routes = graphql_playground
        .or(graphql_post)
        .or(graphql_subscription)
        .or(cors_route)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
