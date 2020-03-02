use crate::{ Context, ResultExt, unauthorized };
use super::Machine;

use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Algorithm, Header, EncodingKey};

#[derive(juniper::GraphQLInputObject)]
pub struct AuthorizeForMachineInput {
    pub identity_public_key: String,
    pub machine_slug: String,
}

#[derive(juniper::GraphQLObject, Debug)]
pub struct AuthorizeForMachineResult {
    pub machine: Machine,
    pub token: String,
}

pub async fn authorize_for_machine(
    context: &Context,
    input: AuthorizeForMachineInput,
) -> crate::Result<AuthorizeForMachineResult> {
    let user_id = context.user_id.ok_or(unauthorized())?;

    let machine = sqlx::query_as!(
        Machine,
        "
            SELECT * FROM machines WHERE user_id=$1 AND slug=$2
        ",
        user_id,
        input.slug
    )
        .fetch_one(&mut context.sqlx_db().await?)
        .await
        .chain_err(|| "Unable to select machine for authorization")?;

    // #[derive(Debug, Serialize, Deserialize)]
    // struct Claims {
    //     sub: String,
    //     machineID: String
    // }

    // let claims = Claims {
    //     sub: "b@b.com".to_owned(),
    //     company: "ACME".to_owned()
    // };


    // let secret = env::var("MACHINE_TOKEN_PRIVATE_KEY")?;
    // if (secret.bytes().length < 4096 / 8) {
    //     panic!("MACHINE_TOKEN_PRIVATE_KEY must be 4096 bytes");
    // }

    // let mut header = Header::default();
    // header.alg = RS384;

    // let token = encode(
    //     &header,
    //     &claims,
    //     &EncodingKey::from_secret(secret.as_ref())
    // )?;

    Ok(AuthorizeForMachineResult {
        machine,
        token: "TODO",
    })
}