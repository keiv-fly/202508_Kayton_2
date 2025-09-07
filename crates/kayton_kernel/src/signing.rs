use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub type HmacSha256 = Hmac<Sha256>;

pub fn sign_bytes(key: &[u8], parts: &[&[u8]]) -> String {
    if key.is_empty() {
        return String::new();
    }
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
    for p in parts {
        mac.update(p);
    }
    let out = mac.finalize().into_bytes();
    hex::encode(out)
}

pub fn validate_signature(
    key: &[u8],
    header_bytes: &[u8],
    parent_bytes: &[u8],
    metadata_bytes: &[u8],
    content_bytes: &[u8],
    signature: &str,
) -> bool {
    let computed = sign_bytes(key, &[header_bytes, parent_bytes, metadata_bytes, content_bytes]);
    computed.eq_ignore_ascii_case(signature)
}
