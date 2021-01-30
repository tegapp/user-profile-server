use std::collections::HashMap;
use serde::Deserialize;
use frank_jwt::{Algorithm, ValidationOptions, decode};
use eyre::{
    eyre,
    Result,
    Context as _,
};

// decode the JWT with the matching signing key and validate the payload
#[derive(Deserialize, Debug)]
pub struct JWTPayload {
    pub sub: String,
    pub aud: String,
    pub email: String,
    pub email_verified: bool,
}

use openssl::x509::X509;

use crate::PemKeyList;

pub struct PemKey(Vec<u8>);

const CERTS_URI: &'static str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

pub async fn get_pem_keys() -> Result<Vec<PemKey>> {
    info!("Downloading Firebase Certs");

    // get the latest signing keys
    let req = surf::get(CERTS_URI)
        .recv_json::<HashMap<String, String>>()
        .await
        .map_err(|err| eyre!(err)) // TODO: Remove me when surf 2.0 is released
        .with_context(|| "Unable to get ice candidates")?;

    let pem_keys = req
        .values()
        .map(|x509| -> Option<PemKey> {
            let pem_key_bytes = X509::from_pem(&x509[..].as_bytes())
                .ok()?
                .public_key()
                .ok()?
                .public_key_to_pem()
                .ok()?;

            Some(PemKey(pem_key_bytes))
        })
        .map(|result|
            result.expect("Unable to parse one of google's PEM keys")
        )
        .collect();

    info!("Downloading Firebase Certs  [DONE]");

    Ok(pem_keys)
}

pub async fn validate_jwt(
    pem_keys: &PemKeyList,
    jwt: String,
) -> Result<JWTPayload> {
    let (_, payload) = pem_keys
        .load()
        .iter()
        .find_map(|pem_key| {
            decode(
                &jwt,
                &pem_key.0,
                Algorithm::RS256,
                &ValidationOptions::default(),
            ).ok()
        })
        .ok_or(eyre!("Invalid authorization token"))?;

    let payload: JWTPayload = serde_json::from_value(payload)
        .wrap_err("Invalid authorization payload")?;

    let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
        .expect("$FIREBASE_PROJECT_ID must be set");

    if payload.aud != firebase_project_id {
        Err(eyre!("Invalid JWT Audience"))?
    }

    Ok(payload)
}
