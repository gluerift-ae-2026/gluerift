use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalError {
    #[error("RFC 8785 canonicalization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn canonical_bytes<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    Ok(serde_jcs::to_vec(value)?)
}

pub fn canonical_sha256<T: Serialize + ?Sized>(value: &T) -> Result<String, CanonicalError> {
    let bytes = canonical_bytes(value)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn canonicalization_is_key_order_independent() {
        let a = serde_json::json!({"z": 1, "a": 2});
        let mut b = BTreeMap::new();
        b.insert("a", 2);
        b.insert("z", 1);
        assert_eq!(canonical_sha256(&a).unwrap(), canonical_sha256(&b).unwrap());
    }
}
