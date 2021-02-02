use openssl::{
    ec::{
        EcKey,
        EcGroup,
    },
    nid::Nid,
};
use eyre::{
    // eyre,
    Result,
    // Context as _,
};

pub fn b58_fingerprint(identity_public_key: &String) -> Result<String> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let mut ctx = openssl::bn::BigNumContext::new().unwrap();

    let public_key_bytes: Vec<u8> = identity_public_key.bytes().collect();
    let key = EcKey::public_key_from_pem(&public_key_bytes[..])?;
    let compressed_public_key = key
        .public_key()
        .to_bytes(&group, openssl::ec::PointConversionForm::COMPRESSED, &mut ctx)?;
    let b58_fingerprint = bs58::encode(compressed_public_key).into_string();

    Ok(b58_fingerprint)
}
