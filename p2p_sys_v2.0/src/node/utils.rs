use sha2::{Sha256, Digest};

/// Hash a given data slice and return a hexadecimal string.
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
