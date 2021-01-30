use eyre::{
    eyre,
    Result,
    Context as _,
};

use crate::PemKeyList;

// use crate::unauthorized;
use super::{User, jwt::{validate_jwt}};

pub async fn authorize_user(
    db: &crate::Db,
    pem_keys: &PemKeyList,
    authorization_header: String,
) -> Result<User> {
    // TODO: parse and verify the authorization token
    if !authorization_header.starts_with("Bearer") {
        Err(eyre!("Invalid authorization header"))?;
    }

    let jwt = authorization_header[7..].to_string();

    let payload = validate_jwt(pem_keys, jwt).await?;

    trace!("payload: {:?}", payload);

    let firebase_uid = payload.sub;

    let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
        .expect("$FIREBASE_PROJECT_ID must be set");

    if payload.aud != firebase_project_id {
        Err(eyre!("Invalid JWT Audience"))?;
    }

    // Upsert the user
    let user = sqlx::query_as!(
        User,
        "
            UPDATE users
            SET
                email=$2,
                email_verified=$3
            WHERE firebase_uid = $1
            RETURNING *
        ",
        firebase_uid,
        payload.email,
        payload.email_verified
    )
        .fetch_optional(db)
        .await
        .wrap_err( "PG error authorizing user")?;

    let user = if let Some(user) = user {
        user
    } else {
        sqlx::query_as!(
            User,
            "
                INSERT INTO users (firebase_uid, email, email_verified)
                VALUES ($1, $2, $3)
                RETURNING *
            ",
            firebase_uid,
            payload.email,
            payload.email_verified
        )
            .fetch_one(db)
            .await
            .wrap_err( "PG error authorizing user")?
    };

    Ok(user)
}
