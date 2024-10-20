use k256::ecdsa::{SigningKey, Signature, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::ecdsa::signature::{Signer, Verifier};
use wasm_bindgen::prelude::*;
use sha256::digest;
use chrono::Utc;

#[wasm_bindgen]
pub fn get_time() -> u64 {

    let now = Utc::now();
    let seconds = now.timestamp();

    seconds as u64
}

fn hex_to_byte(hex: &str) -> Result<Vec<u8>, String> {

    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "Invalid hex string".to_string())
}

#[wasm_bindgen]
pub fn create_key() -> String {

    let priv_key = SigningKey::random(&mut OsRng);
    let pub_key = VerifyingKey::from(&priv_key);
    
    format!("{:x}:{:x}",pub_key.to_bytes(), priv_key.to_bytes())
}

#[wasm_bindgen]
pub fn priv_to_pub(priv_key: String) -> Result<String, String> {

    let priv_key_bytes = hex_to_byte(&priv_key)?;
    let priv_key = SigningKey::from_bytes(&priv_key_bytes)
        .map_err(|_| "Invalid private key".to_string())?;


    let pub_key = VerifyingKey::from(&priv_key);
    
    Ok(format!("{:x}",pub_key.to_bytes()))
}


#[wasm_bindgen]
pub fn sign(priv_key: &str, hash: &str) -> Result<String, String> {

    let priv_key_bytes = hex_to_byte(priv_key)?;
    let priv_key = SigningKey::from_bytes(&priv_key_bytes)
        .map_err(|_| "Invalid private key".to_string())?;

    let hash_bytes = hex_to_byte(hash)?;

    let signature: Signature = priv_key.sign(&hash_bytes);

    Ok(format!("{:x}", signature))
}

#[wasm_bindgen]
pub fn verify(pub_key: &str, hash: &str, signature: &str) -> Result<bool, String> {

    let pub_key_bytes = hex_to_byte(pub_key)?;
    let pub_key = VerifyingKey::from_sec1_bytes(&pub_key_bytes)
        .map_err(|_| "Invalid public key".to_string())?;

    let signature_bytes = hex_to_byte(signature)?;
    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| "Invalid signature".to_string())?;

    let hash_bytes = hex_to_byte(hash)?;

    Ok(pub_key.verify(&hash_bytes, &signature).is_ok())
}

#[wasm_bindgen]
pub fn get_hash(input: &str) -> String {

    let hash = digest(input);

    hash
}

