//! Authentication primitives: Argon2 password hashing and HS256 JWTs.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("password hashing failed")]
    Hash,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid or expired token")]
    InvalidToken,
}

/// JWT claims embedded in issued tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user id).
    pub sub: String,
    pub email: String,
    /// Issued-at (epoch seconds).
    pub iat: usize,
    /// Expiry (epoch seconds).
    pub exp: usize,
}

/// Hash a plaintext password with Argon2id and a random salt.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AuthError::Hash)
}

/// Verify a plaintext password against a stored Argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Issue an HS256 JWT valid for `ttl_secs` seconds.
pub fn issue_token(
    secret: &[u8],
    user_id: &str,
    email: &str,
    ttl_secs: u64,
) -> Result<String, AuthError> {
    let now = unix_now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: now + ttl_secs as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| AuthError::InvalidToken)
}

/// Verify a token's signature and expiry, returning its claims.
pub fn verify_token(secret: &[u8], token: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidToken)
}

fn unix_now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_round_trip() {
        let hash = hash_password("hunter2").unwrap();
        assert!(verify_password("hunter2", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn token_round_trip() {
        let secret = b"test-secret";
        let token = issue_token(secret, "u1", "a@b.com", 60).unwrap();
        let claims = verify_token(secret, &token).unwrap();
        assert_eq!(claims.sub, "u1");
        assert_eq!(claims.email, "a@b.com");
        assert!(verify_token(b"other-secret", &token).is_err());
    }
}
