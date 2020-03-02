use async_trait::async_trait;
use std::sync::Arc;
use crate::ResultExt;

use super::{
    Session,
    SessionStore,
};

pub struct SQLXStore {
    pool: Arc<sqlx::PgPool>,
    secret: String,
}

impl SQLXStore {
    async fn db(&self) -> crate::Result<&mut sqlx::pool::PoolConnection<sqlx::PgConnection>> {
        let db = &mut self.pool
            .acquire()
            .await
            .chain_err(|| "Unable to connect to postgres session store")?;
        Ok(&mut db)
    }
}

#[async_trait]
impl SessionStore for SQLXStore {
    fn secret(&self) -> &str {
        &self.secret
    }

    async fn get(
        &self,
        session_id: String,
    ) -> crate::Result<Option<Session>> {
        let session = sqlx::query!(
            "SELECT * FROM sessions WHERE sid = $1",
            session_id
        )
            .fetch_one(self.db().await?)
            .await
            .chain_err(|| "Unable to select session")?;

        let sess = serde_json::from_str(&session.sess)
            .chain_err(|| "Unable to deserialize session.sess")?;

        let session = Session {
            sid: session.sid,
            sess,
            expire: session.expire,
            secret: self.secret.clone(),
        };

        Ok(Some(session))
    }

    async fn create(
        &self,
        session: Session,
    ) -> crate::Result<()> {
        let sess = serde_json::to_string(&session.sess)
            .chain_err(|| "Unable to deserialize session.sess")?;

        let session = sqlx::query!(
            "
                INSERT INTO sessions (sid, sess, expire)
                VALUES ($1, $2, $3)
            ",
            session.sid,
            sess,
            session.expire
        )
            .fetch_one(self.db().await?)
            .await
            .chain_err(|| "Unable to insert session")?;

        Ok(())

    }
}
