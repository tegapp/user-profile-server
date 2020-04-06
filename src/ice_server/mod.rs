use crate::{ Context, ResultExt, unauthorized };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, juniper::GraphQLObject)]
pub struct IceServer {
    url: Option<String>,
    urls: Option<Vec<String>>,
    username: Option<String>,
    credential: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TwillioResponse {
    ice_servers: Vec<IceServer>
}

fn dev_twillio_mock() -> TwillioResponse {
    TwillioResponse {
        ice_servers: vec![
            IceServer {
                urls: Some(vec!["stun:stun.l.google.com:19302".to_string()]),
                ..Default::default()
            },
            IceServer {
                urls: Some(vec!["stun:global.stun.twilio.com:3478?transport=udp".to_string()]),
                ..Default::default()
            }
        ]
    }
}

pub async fn get_ice_servers(context: &Context) -> crate::Result<Vec<IceServer>> {
    let _ = context.user_id().ok_or(unauthorized())?;

    let rust_env = std::env::var("RUST_ENV").chain_err(|| "RUST_ENV not found")?;

    let res = if rust_env == "development" {
        dev_twillio_mock()
    } else {
        let twilio_sid = std::env::var("TWILIO_SID").chain_err(|| "TWILIO_SID not found")?;
        let twilio_token = std::env::var("TWILIO_TOKEN").chain_err(|| "TWILIO_TOKEN not found")?;

        let uri = format!("https://video.twilio.com/2010-04-01/Accounts/{:}/Tokens", twilio_sid);

        let client = reqwest::Client::new();

        client.get(&uri)
            .basic_auth(twilio_sid, Some(twilio_token))
            .send()
            .await
            .chain_err(|| "Unable to fetch ice server list")?
            .json::<TwillioResponse>()
            .await
            .chain_err(|| "Unable to parse ice server list")?
    };

    Ok(res.ice_servers)
}
