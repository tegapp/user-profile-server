use serde::{Serialize, Deserialize};
use eyre::{
    eyre,
    Result,
    // Context as _,
};

use crate::{machine::Machine, user::User};

pub struct AuthContext {
    user: Option<User>,
    machine: Option<Machine>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WebSocketAuthentication {
    identity_public_key: String,
    #[serde(rename = "selfSignedJWT")]
    self_signed_jwt: String,
}

impl AuthContext {
    pub async fn http_post_auth(
        db: &crate::Db,
        pem_keys: &crate::PemKeyList,
        authorization_header: Option<String>,
    ) -> Result<Self> {
        let user = if let Some(authorization_header) = authorization_header {
            let pem_keys = pem_keys.clone();

            let user = crate::user::authorize_user(
                &db,
                &pem_keys,
                authorization_header,
            ).await?;

            Some(user)
        } else {
            None
        };

        let auth = AuthContext {
            user,
            machine: None,
        };

        Ok(auth)
    }

    pub async fn websocket_auth(
        db: &crate::Db,
        json: serde_json::Value,
    ) -> Result<Self> {
        let WebSocketAuthentication {
            identity_public_key,
            self_signed_jwt,
        } = serde_json::from_value(json)?;

        // TODO: Verify the jwt signature

        // TODO: Look up the machine

        let auth = AuthContext {
            user: None,
            machine: None,
        };

        Ok(auth)
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

    pub fn require_machine(&self) -> Result<&Machine> {
        self.machine
            .as_ref()
            .ok_or_else(||
                eyre!("Not authorized.")
            )
    }
}
