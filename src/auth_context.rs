use serde::{Serialize, Deserialize};
use eyre::{
    eyre,
    Result,
    Context as _,
};
use async_graphql::extensions::TracingConfig;

use crate::{b58_fingerprint, host::Host, user::User};

pub struct AuthContext {
    user: Option<User>,
    host: Option<Host>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct JWTPayload {
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "aud")]
    pub audience: String,
    pub self_signature: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WebSocketAuthentication {
    /// The host's identity public key in PEM format.
    identity_public_key: String,
    #[serde(rename = "selfSignedJWT")]
    /// Verified against the host's identity public key
    self_signed_jwt: String,
}

// lazy_static! {
//     static ref ws_root_span: tracing::span::Span = span!(
//         parent: None,
//         tracing::Level::INFO,
//         "ws root",
//     );

//     static ref req_root_span: tracing::span::Span = span!(
//         parent: None,
//         tracing::Level::INFO,
//         "req root",
//     );
// }

impl AuthContext {
    pub async fn http_post_auth(
        db: crate::Db,
        pem_keys: crate::PemKeyList,
        authorization_header: Option<String>,
        schema: crate::AppSchema,
        request: async_graphql::Request,
    ) -> std::result::Result<async_graphql_warp::Response, warp::Rejection> {
        let user = if let Some(authorization_header) = authorization_header {
            let pem_keys = pem_keys.clone();

            let user = crate::user::authorize_user(
                &db,
                &pem_keys,
                authorization_header,
            ).await;

            let user = match user {
                Ok(user) => user,
                Err(err) => {
                    warn!("{:?}", err);
                    return Err(warp::reject::custom(crate::InternalServerError))
                }
            };

            Some(user)
        } else {
            None
        };

        let auth = AuthContext {
            user,
            host: None,
        };

        let request = request
            .data(auth)
            .data(TracingConfig::default());
            // .data(TracingConfig::default().parent_span(req_root_span.clone()));

        Ok(async_graphql_warp::Response::from(schema.execute(request).await))
    }

    pub async fn websocket_auth(
        db: crate::Db,
        json: serde_json::Value,
    ) -> async_graphql::Result<async_graphql::Data> {
        Self::websocket_auth_inner(db, json)
            .await
            .map_err(|err| {
                warn!("websocket auth error: {:?}", err);
                eyre!("Internal Server Error").into()
            })
    }

    async fn websocket_auth_inner(
        db: crate::Db,
        json: serde_json::Value,
    ) -> Result<async_graphql::Data> {

        let WebSocketAuthentication {
            identity_public_key,
            self_signed_jwt,
        } = serde_json::from_value(json)?;

        // Verify the jwt signature
        let (_, payload) = frank_jwt::decode(
            &self_signed_jwt,
            &identity_public_key,
            frank_jwt::Algorithm::ES256,
            &frank_jwt::ValidationOptions::default(),
        )?;
        // JWT payload validation
        let payload: JWTPayload = serde_json::from_value(payload)
            .wrap_err("Invalid websocket jwt")?;

        if !payload.self_signature {
            Err(eyre!("JWT payload field 'selfSignature' must be true"))?;
        }

        let signalling_url = "https:://signalling.tegapp.com";

        if payload.audience != signalling_url {
            Err(eyre!("Expected JWT aud: {}, got: {}", signalling_url, payload.audience))?;
        }

        // Add the host to the database
        let host = sqlx::query_as!(
            Host,
            "SELECT * FROM hosts WHERE identity_public_key = $1",
            identity_public_key,
        )
            .fetch_optional(&db)
            .await?;

        let host = if let Some(host) = host {
            host
        } else {
            let slug = b58_fingerprint(&identity_public_key)?;

            sqlx::query_as!(
                Host,
                r#"
                    INSERT INTO hosts (identity_public_key, slug)
                    VALUES ($1, $2)
                    RETURNING *
                "#,
                identity_public_key,
                slug,
            )
                .fetch_one(&db)
                .await?
        };

        let auth = AuthContext {
            user: None,
            host: Some(host),
        };

        let mut data = async_graphql::Data::default();
        data.insert(auth);
        data.insert(TracingConfig::default());
        // data.insert(TracingConfig::default().parent_span(ws_root_span.clone()));

        Ok(data)
    }

    pub fn user_id(&self) -> Option<crate::DbId> {
        self.user.as_ref().map(|user| user.id)
    }

    pub fn allow_unauthorized_user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    pub fn require_authorized_user(&self) -> Result<&User> {
        self.user
            .as_ref()
            .ok_or_else(||
                eyre!("Not authorized.")
            )
    }

    pub fn require_host(&self) -> Result<&Host> {
        self.host
            .as_ref()
            .ok_or_else(||
                eyre!("Not authorized.")
            )
    }
}
