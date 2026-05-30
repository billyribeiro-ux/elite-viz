//! Bearer-token authentication extractor.

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use finviz_core::AppState;
use finviz_types::User;

use crate::error::AppError;

/// Extractor that authenticates the request via `Authorization: Bearer <jwt>`
/// and resolves the current [`User`]. Returns 401 on any failure.
pub struct AuthUser(pub User);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("expected a Bearer token".into()))?;
        let claims = finviz_auth::verify_token(state.jwt_secret(), token)
            .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))?;
        let user = state
            .user_by_id(&claims.sub)
            .ok_or_else(|| AppError::Unauthorized("user no longer exists".into()))?;
        Ok(AuthUser(user))
    }
}
