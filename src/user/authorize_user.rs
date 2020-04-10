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
    let mut user = sqlx::query_as!(
        User,
        "
            INSERT INTO users(
                firebase_uid,
                email,
                email_verified
            ) VALUES (
                $1, $2, $3
            )
            ON CONFLICT (firebase_uid) DO NOTHING
            RETURNING *
        ",
        // TODO: upsert params
        firebase_uid,
        payload.email,
        payload.email_verified
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "PG error authorizing user")?
        .ok_or(unauthorized())?;

    if user.email_verified != payload.email_verified {
        user.email_verified = payload.email_verified;

        let _ = sqlx::query_as!(
            User,
            "
                UPDATE users
                SET email_verified=$1
                WHERE firebase_uid = $2
                RETURNING *
            ",
            payload.email_verified,
            firebase_uid,
        )
            .fetch_optional(&mut context.sqlx_db().await?)
            .await
            .chain_err(|| "PG error authorizing user")?
            .ok_or(unauthorized())?;
    }

    Ok(user)
}
