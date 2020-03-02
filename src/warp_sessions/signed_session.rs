use ring::{hmac, rand};

pub fn sign(session_id: &str, secret: &str) -> crate::Result<String> {
    let rng = rand::SystemRandom::new();
    let key = hmac::Key::new(hmac::HMAC_SHA256, &secret)?;

    let signature = hmac::sign(&key, session_id.as_bytes());

    base64::encode_config(
        format!("s:{}.{}", session_id, signature),
        base64::STANDARD_NO_PAD,
    )
}

pub fn unsign(session_cookie: String, secret: &str) -> crate::Result<String> {
    let session_cookie = base64::decode(session_cookie[2..]);

    let session_id = match session_cookie.split(".").collect() {
        [v, _] => Ok(v),
        _ => Err("Invalid session cookie".into()?),
    }?;

    let expected_cookie = sign(&session_id, secret)?;

    if consistenttime::ct_u8_slice_eq(expected_cookie.bytes(), session_cookie.bytes()) {
        Ok(session_id)
    } else  {
        "Invalid session cookie".into()?
    }
}
