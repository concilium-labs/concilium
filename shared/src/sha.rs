use sha2::{Digest, Sha256};

pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().as_slice().try_into().unwrap()
}