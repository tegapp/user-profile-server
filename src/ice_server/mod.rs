use serde::{Deserialize, Serialize};
use surf::http::auth::BasicAuth;
use std::fmt;
use std::marker::PhantomData;
use serde::de;
use serde::de::Deserializer;
use eyre::{
    eyre,
    Result,
    Context as _,
};

fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
            where S: de::SeqAccess<'de>
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

#[derive(async_graphql::SimpleObject, Serialize, Deserialize, Debug, Default, Clone)]
pub struct IceServer {
    url: Option<String>,
    #[serde(deserialize_with = "string_or_seq_string")]
    urls:  Vec<String>,
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
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
            IceServer {
                urls: vec!["stun:global.stun.twilio.com:3478?transport=udp".to_string()],
                ..Default::default()
            }
        ]
    }
}

pub async fn get_ice_servers() -> Result<Vec<IceServer>> {
    // let _ = context.user_id().ok_or(unauthorized())?;

    let rust_env = std::env::var("RUST_ENV")
        .wrap_err( "RUST_ENV not found")?;

    let res = if
        rust_env == "development"
        && std::env::var("TWILIO_SID").is_err()
    {
        dev_twillio_mock()
    } else {
        let twilio_sid = std::env::var("TWILIO_SID")
            .wrap_err( "TWILIO_SID not found")?;
        let twilio_token = std::env::var("TWILIO_TOKEN")
            .wrap_err( "TWILIO_TOKEN not found")?;

        let uri = format!("https://api.twilio.com/2010-04-01/Accounts/{:}/Tokens.json", twilio_sid);

        info!("Downloading WebRTC ICE server list");

        let res = surf::post(&uri)
            .header(
                "Authorization",
                BasicAuth::new(twilio_sid, twilio_token).value(),
            )
            .recv_json::<TwillioResponse>()
            .await
            .map_err(|err| eyre!(err)) // TODO: Remove me when surf 2.0 is released
            .wrap_err( "Unable to fetch WebRTC ICE server list")?;

        info!("Downloading WebRTC ICE server list  [DONE]");

        res
    };

    Ok(res.ice_servers)
}
