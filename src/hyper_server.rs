extern crate futures;
extern crate hyper;
extern crate juniper;
extern crate juniper_hyper;
extern crate pretty_env_logger;

use juniper::{
    http::GraphQLRequest as JuniperGraphQLRequest,
    // serde::Deserialize,
    // DefaultScalarValue,
    // GraphQLType,
    // InputValue,
    // RootNode, ScalarRefValue, ScalarValue,
    // Variables,
};

use hyper::service::{make_service_fn, service_fn};
use {
    hyper::{
        // Miscellaneous types from Hyper for working with HTTP.
        Body, Method, Request, Response, Server, StatusCode,
    },
    // futures03::{
    //     // Extension trait for futures 0.1 futures, adding the `.compat()` method
    //     // which allows us to use `.await` on 0.1 futures.
    //     // compat::Future01CompatExt,
    //     // Extension traits providing additional methods on futures.
    //     // `FutureExt` adds methods that work for all futures, whereas
    //     // `TryFutureExt` adds methods to futures that return `Result` types.
    //     future::{FutureExt, TryFutureExt},
    // },
};
// use crate::futures03::StreamExt;
// use crate::futures03::TryStreamExt;

use std::sync::Arc;
use super::PgPool;

use super::graphql_schema::{ Query, Mutation, Schema };
use super::context::Context;

use std::env;

use super::auth::{ upsert_user };

// #[derive(Debug)]
// enum GraphQLRequestError {
//     BodyHyper(hyper::Error),
//     BodyUtf8(FromUtf8Error),
//     BodyJSONError(SerdeError),
//     Variables(SerdeError),
//     Invalid(String),
// }

// impl fmt::Display for GraphQLRequestError {
//     fn fmt(&self, mut f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             GraphQLRequestError::BodyHyper(ref err) => fmt::Display::fmt(err, &mut f),
//             GraphQLRequestError::BodyUtf8(ref err) => fmt::Display::fmt(err, &mut f),
//             GraphQLRequestError::BodyJSONError(ref err) => fmt::Display::fmt(err, &mut f),
//             GraphQLRequestError::Variables(ref err) => fmt::Display::fmt(err, &mut f),
//             GraphQLRequestError::Invalid(ref err) => fmt::Display::fmt(err, &mut f),
//         }
//     }
// }

// impl Error for GraphQLRequestError {
//     fn description(&self) -> &str {
//         match *self {
//             GraphQLRequestError::BodyHyper(ref err) => err.description(),
//             GraphQLRequestError::BodyUtf8(ref err) => err.description(),
//             GraphQLRequestError::BodyJSONError(ref err) => err.description(),
//             GraphQLRequestError::Variables(ref err) => err.description(),
//             GraphQLRequestError::Invalid(ref err) => err,
//         }
//     }

//     fn cause(&self) -> Option<&dyn Error> {
//         match *self {
//             GraphQLRequestError::BodyHyper(ref err) => Some(err),
//             GraphQLRequestError::BodyUtf8(ref err) => Some(err),
//             GraphQLRequestError::BodyJSONError(ref err) => Some(err),
//             GraphQLRequestError::Variables(ref err) => Some(err),
//             GraphQLRequestError::Invalid(_) => None,
//         }
//     }
// }

fn server_error (message: &str) -> Response<Body> {
    let mut resp = Response::new(Body::empty());
    use hyper::header::{
        ACCESS_CONTROL_ALLOW_ORIGIN,
    };

    resp.headers_mut().insert(
        ACCESS_CONTROL_ALLOW_ORIGIN,
        "*".parse().unwrap(),
    );

    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    *resp.body_mut() = Body::from(message.to_string());
    resp
}

async fn serve_req<'a>(
    req: Request<Body>,
    root_node: Arc<juniper::RootNode<'_, Query, Mutation>>,
    pool: PgPool,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            // let response = juniper_hyper::graphiql("/graphql").compat().await;
            let mut response = Response::new(Body::empty());
            *response.body_mut() = Body::from(juniper::graphiql::graphiql_source("/graphql"));
            // *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
        (&Method::OPTIONS, _) => {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::NO_CONTENT;

            use hyper::header::{
                ACCESS_CONTROL_ALLOW_ORIGIN,
                ACCESS_CONTROL_ALLOW_METHODS,
                ACCESS_CONTROL_ALLOW_HEADERS,
                ACCESS_CONTROL_MAX_AGE,
            };

            resp.headers_mut().insert(
                ACCESS_CONTROL_ALLOW_ORIGIN,
                "*".parse().unwrap(),
            );
            resp.headers_mut().insert(
                ACCESS_CONTROL_ALLOW_METHODS,
                "POST, GET, OPTIONS, DELETE".parse().unwrap(),
            );
            resp.headers_mut().insert(
                ACCESS_CONTROL_ALLOW_HEADERS,
                "*".parse().unwrap(),
            );
            resp.headers_mut().insert(
                ACCESS_CONTROL_MAX_AGE,
                "86400".parse().unwrap(),
            );

            Ok(resp)
        }
        (&Method::POST, "/graphql") => {
            let user = upsert_user(
                Arc::clone(&pool),
                req,
            ).await;

            if let Ok((user, req)) = user {
                let root_node = root_node.clone();

                // let root_node = Arc::new(Schema::new(Query, Mutation{}));


                let ctx = Arc::new(Context {
                    pool,
                    user_id: user.id,
                });

                let body = hyper::body::to_bytes(req.into_body()).await;

                if let Ok(body) = body {
                    // let mut query = None;
                    // let operation_name = None;
                    // let mut variables = None;

                    // for (key, value) in url::form_urlencoded::parse(&body).into_owned() {
                    //     match key.as_ref() {
                    //         "query" => {
                    //             if query.is_some() {
                    //                 return Ok(server_error("cannot set query twice"));
                    //             }
                    //             query = Some(value)
                    //         }
                    //         "operationName" => {
                    //             if operation_name.is_some() {
                    //                 return Ok(server_error("cannot set operationName twice"));
                    //             }
                    //         }
                    //         "variables" => {
                    //             if variables.is_some() {
                    //                 return Ok(server_error("cannot set variables twice"));
                    //             }
                    //             match serde_json::from_str::<InputValue<DefaultScalarValue>>(&value)
                    //             {
                    //                 Ok(parsed_variables) => variables = Some(parsed_variables),
                    //                 Err(_) => {
                    //                     return Ok(server_error("invalid variables"));
                    //                 },
                    //             }
                    //         }
                    //         _ => continue,
                    //     }
                    // };

                    // let variables = variables
                    //     .and_then(|v| v.to_object_value())
                    //     .map(|v| Variables::new(v))
                    //     .unwrap_or(Variables::new());

                    let graphql_req = serde_json::from_slice::<JuniperGraphQLRequest>(&body);

                    match graphql_req {
                        Ok(graphql_req) => {
                            // let graphql_req = JuniperGraphQLRequest::new(query, operation_name, variables);
                            
                            let graphql_result = graphql_req.execute(
                                &root_node, 
                                &ctx,
                            );

                            let code = if graphql_result.is_ok() {
                                StatusCode::OK
                            } else {
                                StatusCode::BAD_REQUEST
                            };

                            let mut resp = Response::new(Body::empty());
                            *resp.status_mut() = code;

                            use hyper::header::{
                                CONTENT_TYPE,
                                ACCESS_CONTROL_ALLOW_ORIGIN,
                            };

                            resp.headers_mut().insert(
                                ACCESS_CONTROL_ALLOW_ORIGIN,
                                "*".parse().unwrap(),
                            );
                
                            resp.headers_mut().insert(
                                CONTENT_TYPE,
                                "application/json".parse().unwrap(),
                            );

                            let body = Body::from(serde_json::to_string_pretty(&graphql_result).unwrap());
                            *resp.body_mut() = body;

                            Ok(resp)

                            // let result = juniper::execute(
                            //     &query, 
                            //     None, 
                            //     &root_node, 
                            //     &variables,
                            //     &ctx,
                            // );
                        },
                        Err(err) => {
                            return Ok(server_error(&err.to_string()));
                        },
                    }
                } else {
                    Ok(server_error("Invalid form post"))
                }
            } else {
                println!("Unauthorized {:?}", user);

                let mut resp = Response::new(Body::empty());

                use hyper::header::{
                    ACCESS_CONTROL_ALLOW_ORIGIN,
                };

                resp.headers_mut().insert(
                    ACCESS_CONTROL_ALLOW_ORIGIN,
                    "*".parse().unwrap(),
                );
    

                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                Ok(resp)
            }
        }
        _ => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}

pub async fn run(pool: PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let port = env::var("PORT")
        .expect("$PORT must be set")
        .parse()
        .expect("Invalid $PORT");

    let addr = ([0, 0, 0, 0], port).into();

    // let pool = Arc::new(pool);
    let root_node = Arc::new(Schema::new(Query, Mutation{}));

    let make_svc = make_service_fn(move |_| {
        let root_node = Arc::clone(&root_node);
        let pool = Arc::clone(&pool);
        // let pool = pool.clone();

        async {
            // let root_node = Arc::clone(&root_node);
            // let pool = Arc::clone(&pool);
            // let pool = pool.clone();
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let root_node = Arc::clone(&root_node);
                let pool = Arc::clone(&pool);

                serve_req(req, root_node, pool)
            }))
            // serve_req2(req).boxed().compat()
        }
    });

    // Create a server bound on the provided address
    let serve_future = Server::bind(&addr)
        // Serve requests using our `async serve_req` function.
        // `serve` takes a closure which returns a type implementing the
        // `Service` trait. `service_fn` returns a value implementing the
        // `Service` trait, and accepts a closure which goes from request
        // to a future of the response. To use our `serve_req` function with
        // Hyper, we have to box it and put it in a compatability
        // wrapper to go from a futures 0.3 future (the kind returned by
        // `async fn`) to a futures 0.1 future (the kind used by Hyper).
        .serve(make_svc);

    println!("Listening on http://{}", addr);

    // Wait for the server to complete serving or exit with an error.
    // If an error occurred, print it to stderr.
    serve_future.await?;

    Ok(())
}
