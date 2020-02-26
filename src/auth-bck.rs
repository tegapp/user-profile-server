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

// use alcoholic_jwt::{JWKS, Validation, validate, token_kid, ValidJWT};

// use std::sync::Arc;
use super::PgPool;

use super::user::{ NewUser, User };

use serde::{ Deserialize };

#[derive(Deserialize, Debug)]
pub struct Auth0User {
    sub: String,
    name: Option<String>,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    phone_number: Option<String>,
    #[serde(default)]
    phone_number_verified: bool,
}

pub async fn validate_auth0_user(
    bearer_auth: &str,
) -> Result<Auth0User, String> {
    use reqwest::header::{ HeaderMap };

    let auth0_url = "thirtybots-dev.eu.auth0.com".to_string();
    let auth0_url = format!("https://{}/userinfo", auth0_url);

    let bearer_auth = bearer_auth.parse()
        .map_err(|_| "Invalid Bearer Authentication".to_string())?;

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, bearer_auth);

    let res = reqwest::Client::new()
        .post(&auth0_url)
        .headers(headers)
        .send()
        .await
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?;

    // println!("response body: {:?}", res.text().await);
    // Err("".to_string())

    let auth0_user = res
        .json::<Auth0User>()
        .await
        .map_err(|err| err.to_string())?;

    Ok(auth0_user)
}

pub async fn upsert_user(
    pool: PgPool,
    authorization_header: Option<String>,
) -> Result<User, String> {
    // println!("AUTH: {:?}", bearer_auth);

    let authorization_header = authorization_header.ok_or("Missing authorization header")?;

    let auth0_user = validate_auth0_user(&authorization_header).await?;

    use crate::diesel::RunQueryDsl;
    use super::schema::users;
    use super::schema::users::dsl;

    // println!("validated auth0 user: {:?}", auth0_user);

    let new_user = NewUser {
        auth0_id: &auth0_user.sub.as_str(),
        name: auth0_user.name,
        email: auth0_user.email,
        email_verified: auth0_user.email_verified,
        phone_number: auth0_user.phone_number,
        phone_number_verified: auth0_user.phone_number_verified,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .on_conflict(dsl::auth0_id)
        .do_update()
        .set(&new_user)
        .get_result(&pool.get().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    Ok(user)
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