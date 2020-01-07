// extern crate futures;
// extern crate hyper;
// extern crate juniper;
// extern crate juniper_hyper;
// extern crate pretty_env_logger;


// use std::default::Default;
// use crypto::sha2::Sha256;
// use jwt::{
//     Header,
//     Registered,
//     Token,
// };

use hyper::header::{AUTHORIZATION};

// use alcoholic_jwt::{JWKS, Validation, validate, token_kid, ValidJWT};

// use std::sync::Arc;
use super::PgPool;

use super::user::{ NewUser, User };

use serde::{ Deserialize };

#[derive(Deserialize, Debug)]
pub struct Auth0User {
    sub: String,
}

pub async fn validate_auth0_user(
    bearer_auth: &str,
) -> Result<Auth0User, String> {
    use reqwest::header::{ HeaderMap, AUTHORIZATION };

    let auth0_url = "thirtybots-dev.eu.auth0.com".to_string();
    let auth0_url = format!("https://{}/userinfo", auth0_url);

    let bearer_auth = bearer_auth.parse()
        .map_err(|_| "Invalid Bearer Authentication".to_string())?;

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, bearer_auth);

    let auth0_user = reqwest::Client::new()
        .post(&auth0_url)
        .headers(headers)
        .send()
        .await
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?
        .json::<Auth0User>()
        .await
        .map_err(|err| err.to_string())?;

    println!("auth0 response: {:#?}", auth0_user);

    Ok(auth0_user)
}

pub async fn upsert_user(
    pool: PgPool,
    req: hyper::Request<hyper::Body>,
) -> Result<(User, hyper::Request<hyper::Body>), String> {
    let auth_header = req.headers().get(AUTHORIZATION);

    let bearer_auth = auth_header
        .ok_or("Missing Bearer Authentication".to_string())?
        .to_str()
        .map_err(|_| "Invalid Bearer Authentication".to_string())?;

    let auth0_user = validate_auth0_user(bearer_auth).await?;

    use crate::diesel::RunQueryDsl;
    use super::schema::users;

    println!("validated auth0 user: {:?}", auth0_user);

    let new_user = NewUser {
        auth0_id: &auth0_user.sub.as_str(),
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&pool.get().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    Ok((user, req))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_auth() {
        use tokio::runtime::Runtime;

        // Create the runtime
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            let token = std::env::var("AUTH0_TOKEN")
                .expect("Missing auth0 test token. Please use `AUTH0_TOKEN=\"my_token\" cargo test -- --nocapture`");
            let token = format!("Bearer {}", token);
            println!("Token: {}", token);

            let result = validate_auth0_user(token.as_str()).await;

            if let Ok(inner) = result {
                println!("Result: {:?}", inner)
            } else {
                assert!(false, "auth0 call returned an error {:?}", result);
            }
        });
    }

    #[test]
    fn test_bad_auth() {
        use tokio::runtime::Runtime;

        // Create the runtime
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            let token = format!("Bearer aaa111bbb");

            let result = validate_auth0_user(token.as_str()).await;

            // println!("bad auth! {:?}", result);
            assert!(result.is_err(), "bad auth did not error {:?}", result);
        });
    }
}