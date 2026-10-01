use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::phc::PasswordHash;

pub fn hash_password(password: &str) -> String {
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes())
        .expect("Error while hashing password")
        .to_string()
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hashed_password).expect("Invalid hash format");
    let argon2 = Argon2::default();

    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("S3cret-Pass!");
        assert!(hash.starts_with("$argon2"));
        assert!(verify_password("S3cret-Pass!", &hash));
    }

    #[test]
    fn verify_rejects_wrong_password() {
        let hash = hash_password("correct");
        assert!(!verify_password("incorrect", &hash));
    }

    #[test]
    fn hashes_are_salted_and_unique() {
        let h1 = hash_password("same");
        let h2 = hash_password("same");
        assert_ne!(h1, h2);
    }

    #[test]
    #[should_panic]
    fn verify_panics_on_invalid_hash_format() {
        verify_password("whatever", "not-a-valid-hash");
    }
}
