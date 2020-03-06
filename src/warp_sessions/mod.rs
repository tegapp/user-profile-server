use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use chrono::{ DateTime, Duration, Utc };

mod filters;
pub use filters::*;

mod signed_session;
pub use signed_session::*;

mod sqlx_store;
pub use sqlx_store::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    user_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sess
{
    original_max_age: u32,
    http_only: bool,
    path: String,
    csrf_secret: Vec<u8>,
    data: Option<Box<Data>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session
{
    sid: String,
    sess: Sess,
    expire: DateTime<Utc>,
    secret: String,
}

impl Session {
    pub fn cookie_builder(&self) -> cookie::CookieBuilder {
        use cookie::Cookie;

        let Sess {
            original_max_age,
            expires,
            http_only,
            path,
        } = self.sess;

        let value = signed_session::sign(self.sid, &self.secret)?;

        let cookie_builder: Cookie = Cookie::build("connect.sid", value)
            .path("/")
            .http_only(true)
            .max_age(Duration::milliseconds(original_max_age))
            .expires(expires);

        Ok(cookie_builder)
    }
}

#[async_trait]
pub trait SessionStore: Sized + Send + Sync {
    fn secret(&self) -> &str;

    async fn get<'a>(&self, session_id: String) -> crate::Result<Option<Session>>;
    async fn create<'a>(&self, session: &Session) -> crate::Result<()>;
}
