use std::collections::HashMap;
use frank_jwt::{Algorithm, encode, decode};

use crate::{ Context, ResultExt, unauthorized };
use super::User;

pub async fn authorize_user(
    context: &Context,
    authorization_header: String,
) -> Result<User, Box<dyn Error>> {
    // TODO: parse and verify the authorization token
    if !authorization_header.starts_with("Bearer") {
        Err("Invalid authorization header")?
    }

    let jwt = authorization_header[6..];

    // get the latest signing keys
    let uri = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
    let pem_keys: HashMap<String, String> = context.surf.get(uri).recv_json().await?;

    // decode the JWT with the matching signing key and validate the payload
    let (header, payload) = pem_keys.values().find_map(|pem_key| {
        decode(
            &jwt,
            &pem_key,
            Algorithm::RS256,
            &ValidationOptions::default(),
        ).ok()
    }).ok_or("Invalid authorization token")?;

    let jwt_audience = payload.get("aud").ok_or("Missing aud in JWT")?;

    let firebase_project_id = env::var("FIREBASE_PROJECT_ID")
        .expect("$FIREBASE_PROJECT_ID must be set");

    if (jwt_audience != firebase_project_id) {
        Err("Invalid JWT Audience")?
    }

    let firebase_uid = payload.get("sub").ok_or("Missing sub in JWT")?;

    // Upsert the user
    // TODO: how do we get the user's email or any identifier?
    let user = sqlx::query_as!(
        User,
        "
            INSERT INTO users(
                firebase_uid,
                name,
                email,
                email_verified,
                phone_number,
                phone_number_verified
            ) VALUES (
                $1, $2, $3, $4, $5, $6
            )
            ON CONFLICT (firebase_uid) DO UPDATE
            SET
                name=$2
                email=$3
                email_verified=$4
                phone_number=$5
                phone_number_verified=$6
            RETURNING *
        ",
        // TODO: upsert params
        [
            firebase_uid,
        ]
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "PG error authorizing user")?
        .ok_or(unauthorized())?;

    Ok(user)
}