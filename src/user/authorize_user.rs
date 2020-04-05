use crate::{ Context, ResultExt, unauthorized };
use super::{User, jwt::validate_jwt};

pub async fn authorize_user(
    context: &Context,
    authorization_header: String,
) -> Result<User, crate::Error> {
    // TODO: parse and verify the authorization token
    if !authorization_header.starts_with("Bearer") {
        Err("Invalid authorization header")?
    }

    let jwt = authorization_header[7..].to_string();

    let payload = validate_jwt(context, jwt).await?;

    trace!("payload: {:?}", payload);

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
