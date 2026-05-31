//! Runtime configuration, sourced from the environment.

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    /// Address the HTTP server binds to (`BIND_ADDR`, default `0.0.0.0:8080`).
    pub bind_addr: SocketAddr,
    /// Allowed CORS origin (`CORS_ORIGIN`, default `http://localhost:5173`).
    pub cors_origin: String,
    /// Secret used to sign JWTs (`JWT_SECRET`).
    ///
    /// Note: [`crate::AppState::seeded`] reads `JWT_SECRET` from the environment
    /// directly (with the same default), so the server's signing key tracks the
    /// environment regardless of this field. It is exposed here for callers that
    /// want the resolved value without re-reading the environment.
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("BIND_ADDR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8080)));

        let cors_origin =
            std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());

        let jwt_secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

        Self {
            bind_addr,
            cors_origin,
            jwt_secret,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}
