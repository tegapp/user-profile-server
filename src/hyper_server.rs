extern crate futures;
extern crate hyper;
extern crate juniper;
extern crate juniper_hyper;
extern crate pretty_env_logger;

use futures::future;
use hyper::{
    rt::{self, Future},
    service::service_fn,
    Body, Method, Response, Server, StatusCode,
};

use std::sync::Arc;
use super::PgPool;

use futures03::future::{FutureExt, TryFutureExt};

use super::graphql_schema::{ Query, Mutation, Schema };
use super::context::Context;

use super::auth::{ upsert_user };

pub fn run(pool: PgPool) {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    // let pool = Arc::new(pool);
    let root_node = Arc::new(Schema::new(Query, Mutation{}));

    let new_service = move || {
        let pool = pool.clone();
        let root_node = root_node.clone();

        service_fn(move |req| -> Box<dyn Future<Item = _, Error = _> + Send> {
            let root_node = root_node.clone();
            let pool = pool.clone();

            let user = upsert_user(
                pool.clone(),
                req,
            );
            let user = user.boxed().compat();

            let result = user.then(move |user| -> Box<dyn Future<Item = _, Error = _> + Send> {
                if let Ok((user, req)) = user {
                    let root_node = root_node.clone();

                    let ctx = Arc::new(Context {
                        pool,
                        user_id: user.id,
                    });

                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => Box::new(juniper_hyper::graphiql("/graphql")),
                        (&Method::GET, "/graphql") => Box::new(juniper_hyper::graphql(root_node, ctx, req)),
                        (&Method::POST, "/graphql") => {
                            Box::new(juniper_hyper::graphql(root_node, ctx, req))
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            Box::new(future::ok(response))
                        }
                    }
                } else {
                    let mut response = Response::new(Body::empty());
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                    Box::new(future::ok(response))
                }
            });

            Box::new(result)
        })
    };
    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on http://{}", addr);

    rt::run(server);
}
