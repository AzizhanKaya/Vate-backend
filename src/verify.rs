use k256::ecdsa::{SigningKey, Signature, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::ecdsa::signature::{Signer, Verifier};

fn hex_to_byte(hex: &str) -> Result<Vec<u8>, String> {

    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "Invalid hex string".to_string())
}

pub fn create_key() -> (VerifyingKey, SigningKey) {

    let priv_key = SigningKey::random(&mut OsRng);
    let pub_key = VerifyingKey::from(&priv_key);
    
    (pub_key, priv_key)
}

pub fn priv_to_pub(priv_key: &str) -> Result<VerifyingKey, String> {

    let priv_key_bytes = hex_to_byte(priv_key)?;
    let priv_key = SigningKey::from_bytes(&priv_key_bytes)
        .map_err(|_| "Invalid private key".to_string())?;
    let pub_key = VerifyingKey::from(&priv_key);
    
    Ok(pub_key)
}

pub fn sign(priv_key: &str, hash: &str) -> Result<Signature, String> {

    let priv_key_bytes = hex_to_byte(priv_key)?;
    let priv_key = SigningKey::from_bytes(&priv_key_bytes)
        .map_err(|_| "Invalid private key".to_string())?;

    let hash_bytes = hex_to_byte(hash)?;

    let signature = priv_key.sign(&hash_bytes);

    Ok(signature)
}

pub fn verify(pub_key: &str, hash: String, signature: &str) -> Result<bool, String> {

    let pub_key_bytes = hex_to_byte(pub_key)?;
    let pub_key = VerifyingKey::from_sec1_bytes(&pub_key_bytes)
        .map_err(|_| "Invalid public key".to_string())?;

    let signature_bytes = hex_to_byte(signature)?;
    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| "Invalid signature".to_string())?;

    let hash_bytes = hex_to_byte(hash.as_str())?;

    Ok(pub_key.verify(&hash_bytes, &signature).is_ok())
}
