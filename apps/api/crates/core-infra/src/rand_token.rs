//! CSPRNG-backed session ids and API tokens. Randomness comes from the OS via
//! argon2's re-exported `rand_core::OsRng`, and tokens are hashed with SHA-256
//! for storage.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use core_domain::ids::SessionId;
use core_domain::ports::{ApiToken, TokenGenerator};
use sha2::{Digest, Sha256};

#[derive(Default, Clone)]
pub struct RandTokenGenerator;

impl RandTokenGenerator {
    fn random_bytes<const N: usize>() -> [u8; N] {
        let mut buf = [0u8; N];
        // OS CSPRNG; on the practically-impossible failure, fall back to a
        // panic rather than emit a low-entropy token.
        getrandom::fill(&mut buf).expect("OS RNG must be available");
        buf
    }
}

impl TokenGenerator for RandTokenGenerator {
    fn new_session_id(&self) -> SessionId {
        // 32 bytes ≈ 256 bits of entropy.
        let bytes = Self::random_bytes::<32>();
        SessionId::new(URL_SAFE_NO_PAD.encode(bytes))
    }

    fn new_api_token(&self) -> ApiToken {
        let bytes = Self::random_bytes::<24>();
        let secret = URL_SAFE_NO_PAD.encode(bytes);
        let full = format!("ms_live_{secret}");
        // Prefix shown in the UI: "ms_live_" + first 4 secret chars.
        let prefix: String = full.chars().take(12).collect();
        ApiToken { full, prefix }
    }

    fn hash_api_token(&self, full: &str) -> String {
        let digest = Sha256::digest(full.as_bytes());
        hex(&digest)
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ids_are_unique_and_long() {
        let g = RandTokenGenerator;
        let a = g.new_session_id();
        let b = g.new_session_id();
        assert_ne!(a, b);
        assert!(a.as_str().len() >= 40);
    }

    #[test]
    fn api_token_prefix_matches_full() {
        let g = RandTokenGenerator;
        let t = g.new_api_token();
        assert!(t.full.starts_with("ms_live_"));
        assert!(t.full.starts_with(&t.prefix));
        // hashing is stable
        assert_eq!(g.hash_api_token(&t.full), g.hash_api_token(&t.full));
    }
}
