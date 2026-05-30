//! Argon2id password hashing. Uses argon2's re-exported `password_hash` types
//! and CSPRNG, so we don't add a second `password-hash`/`rand_core` version.

use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString};
use argon2::Argon2;
use core_domain::error::DomainError;
use core_domain::ports::PasswordHasher;

#[derive(Default, Clone)]
pub struct Argon2Hasher;

impl PasswordHasher for Argon2Hasher {
    fn hash(&self, plaintext: &str) -> Result<String, DomainError> {
        // 16 bytes of OS entropy, b64-encoded into a PHC salt.
        let mut salt_bytes = [0u8; 16];
        getrandom::fill(&mut salt_bytes)
            .map_err(|e| DomainError::Repo(format!("rng failed: {e}")))?;
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| DomainError::Repo(format!("salt encode failed: {e}")))?;
        Argon2::default()
            .hash_password(plaintext.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| DomainError::Repo(format!("hashing failed: {e}")))
    }

    fn verify(&self, plaintext: &str, phc: &str) -> Result<bool, DomainError> {
        let parsed =
            PasswordHash::new(phc).map_err(|e| DomainError::Repo(format!("bad hash: {e}")))?;
        match Argon2::default().verify_password(plaintext.as_bytes(), &parsed) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(DomainError::Repo(format!("verify failed: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_roundtrip() {
        let h = Argon2Hasher;
        let phc = h.hash("correct horse battery staple").unwrap();
        assert!(phc.starts_with("$argon2"));
        assert!(h.verify("correct horse battery staple", &phc).unwrap());
        assert!(!h.verify("wrong password", &phc).unwrap());
    }
}
