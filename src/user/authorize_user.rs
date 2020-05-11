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
        "SELECT * FROM users WHERE firebase_uid=$1",
        firebase_uid,
    )
        .fetch_optional(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "PG error authorizing user")?;

    let user = if let Some(mut user) = user {
        if user.email_verified != payload.email_verified || user.email != payload.email {
            user.email = payload.email.to_owned();
            user.email_verified = payload.email_verified;

            let _ = sqlx::query_as!(
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
                .fetch_optional(&mut context.sqlx_db().await?)
                .await
                .chain_err(|| "PG error authorizing user")?
                .ok_or(unauthorized())?;
        }

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
            .fetch_one(&mut context.sqlx_db().await?)
            .await
            .chain_err(|| "PG error authorizing user")?
    };

    Ok(user)
}
