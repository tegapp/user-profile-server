use std::collections::HashMap;
use serde::Deserialize;

use frank_jwt::{Algorithm, ValidationOptions, decode};
use openssl::x509::X509;

use crate::{ Context, ResultExt, unauthorized };
use super::User;

pub async fn authorize_user(
    context: &Context,
    authorization_header: String,
) -> Result<User, crate::Error> {
    // TODO: parse and verify the authorization token
    if !authorization_header.starts_with("Bearer") {
        Err("Invalid authorization header")?
    }

    let jwt = &authorization_header[7..];
    println!("{:?}", jwt);

    // get the latest signing keys
    let uri = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

    // let pem_keys: HashMap<String, String> = context.surf
    //     .get(uri)
    //     .recv_json()
    //     .await
    //     .map_err(|_| "Unable to fetch google PEM keys")?;

    let pem_keys = reqwest::get(uri)
        .await
        .map_err(|_| "Unable to fetch google PEM keys")?
        .json::<HashMap<String, String>>()
        .await
        .map_err(|_| "Unable to parse google PEM keys")?;

    // decode the JWT with the matching signing key and validate the payload
    #[derive(Deserialize, Debug)]
    struct JWTPayload {
        pub sub: String,
        pub aud: String,
        pub name: String,
        pub email: String,
        pub email_verified: bool,
    };

    let (_, payload) = pem_keys.values().find_map(|x509| {
        let pem_key = X509::from_pem(&x509[..].as_bytes())
            .ok()?
            .public_key()
            .ok()?
            .public_key_to_pem()
            .ok()?;

        decode(
            jwt,
            &pem_key,
            Algorithm::RS256,
            &ValidationOptions::default(),
        ).ok()
    }).ok_or("Invalid authorization token")?;

    println!("{:?}", payload);

    let payload: JWTPayload = serde_json::from_value(payload)
        .chain_err(|| "Invalid authorization payload")?;

    let firebase_uid = payload.sub;

    let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
        .expect("$FIREBASE_PROJECT_ID must be set");

    if payload.aud != firebase_project_id {
        Err("Invalid JWT Audience")?
    }

    // Upsert the user
    let user = sqlx::query_as!(
        User,
        "
            INSERT INTO users(
                firebase_uid,
                name,
                email,
                email_verified
            ) VALUES (
                $1, $2, $3, $4
            )
            ON CONFLICT (firebase_uid) DO UPDATE
            SET
                name=$2,
                email=$3,
                email_verified=$4
            RETURNING *
        ",
        // TODO: upsert params
        firebase_uid,
        payload.name,
        payload.email,
        payload.email_verified
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "PG error authorizing user")?
        .ok_or(unauthorized())?;

    Ok(user)
}