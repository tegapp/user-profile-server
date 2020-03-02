use warp::{ Filter, filters::BoxedFilter };
use chrono::prelude::*;

use super::{
    Session,
    Sess,
    signed_session,
    SessionStore,
};

async fn session_filter<S: SessionStore>(
    store: S,
    session_cookie: Option<String>,
) -> crate::Result<Option<Session>> {
    let secret = store.secret().to_string();

    let session = if let Some(session_cookie) = session_cookie {
        let session_id = signed_session::unsign(session_cookie, &secret)?;

        let session = store.get(session_id).await?;

        Some(session)
    } else {
        None
    };

    match session {
        Some(session) => Ok(session),
        None => {
            let max_age = chrono::Duration::days(3);

            use rand::{thread_rng, Rng};

            let mut sid = [0u8; 32];
            thread_rng().fill(&mut sid);
            let sid = base64::encode(&sid);

            let mut csrf_secret = [0u8; 32];
            thread_rng().fill(&mut csrf_secret);
            let csrf_secret = csrf_secret.to_vec();

            let sess = Sess {
                original_max_age: max_age.num_milliseconds() as u32,
                http_only: true,
                path: "/".to_string(),
                csrf_secret,
                data: None
            };

            let session = Session {
                sid,
                sess,
                expire: Utc::now() + max_age,
                secret,
            };

            store.create(&session).await?;

            Ok(Some(session))
        }
    }
}

pub fn session<S, F>(
    store: S,
) -> BoxedFilter<(Session,)>
where
    S: SessionStore,
{
    warp::filters::cookie::optional("session")
        .and_then(move |session_cookie| {
            session_filter(store, session_cookie)
        })
        .boxed()
}

fn validate_csrf(
    session: &Session,
    csrf_token: String,
) -> crate::Result<()> {
    let expected_csrf = session.sess.csrf_secret.bytes();

    if consistenttime::ct_u8_slice_eq(csrf_token.bytes(), expected_csrf) {
        Ok(())
    } else {
        Err("Invalid CSRF".into::<crate::Error>())
    }
}

pub type CSRFToken = String;

async fn required_csrf_filter(
    session: Session,
    csrf_token: String
) -> crate::Result<(Session, CSRFToken)> {
    validate_csrf(&session, csrf_token)?;

    Ok((session, csrf_token))
}

pub fn csrf_protected_session<S: SessionStore>(
    store: S,
) -> BoxedFilter<(Session, CSRFToken)> {
    session(store)
        .and(warp::header::<String>("CSRF-Token"))
        .and_then(required_csrf_filter)
        .boxed()
}

async fn optional_csrf_filter(
    session: Session,
    csrf_token: Option<String>
) -> crate::Result<(Session, Option<CSRFToken>)> {
    if let Some(csrf_token) = csrf_token {
        validate_csrf(&session, csrf_token)?;
    };

    Ok((session, csrf_token))
}

pub fn optional_csrf_session<S: SessionStore>(
    store: S,
) -> BoxedFilter<(Session, Option<CSRFToken>)> {
    session(store)
        .and(warp::header::optional::<String>("CSRF-Token"))
        .and_then(optional_csrf_filter)
        .boxed()
}
