use base64::{engine::general_purpose, Engine as _};
use rand_core::Rng;

/// Cryptographically random token: 48 random bytes encoded as unpadded URL-safe base64 (64 chars).
pub fn generate_token() -> String {
    let mut bytes = [0u8; 48];
    rand::rng().fill_bytes(&mut bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_returns_non_empty_string() {
        let token = generate_token();
        assert!(!token.is_empty());
    }

    #[test]
    fn generate_token_has_expected_length() {
        let token = generate_token();
        // 48 bytes base64url without padding = 64 chars
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn generate_token_returns_unique_values() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn generate_token_uses_url_safe_alphabet() {
        let token = generate_token();
        assert!(token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }
}
