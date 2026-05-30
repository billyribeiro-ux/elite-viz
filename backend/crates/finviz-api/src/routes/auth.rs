//! `/api/v1/auth/*` — register, login, current user, token refresh.

use axum::extract::State;
use axum::Json;
use finviz_core::AppState;
use finviz_types::User;
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::error::{ApiResult, AppError};

/// Token lifetime: 24 hours.
const TTL_SECS: u64 = 60 * 60 * 24;

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

fn validate(c: &Credentials) -> Result<(), AppError> {
    if !c.email.contains('@') {
        return Err(AppError::BadRequest("a valid email is required".into()));
    }
    if c.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

fn issue(state: &AppState, user: &User) -> Result<String, AppError> {
    finviz_auth::issue_token(state.jwt_secret(), &user.id, &user.email, TTL_SECS)
        .map_err(|_| AppError::Internal("failed to issue token".into()))
}

pub async fn register(
    State(state): State<AppState>,
    Json(creds): Json<Credentials>,
) -> ApiResult<Json<AuthResponse>> {
    validate(&creds)?;
    let hash = finviz_auth::hash_password(&creds.password)
        .map_err(|_| AppError::Internal("failed to hash password".into()))?;
    let user = state
        .create_user(&creds.email, hash)
        .ok_or_else(|| AppError::Conflict("email is already registered".into()))?;
    let token = issue(&state, &user)?;
    Ok(Json(AuthResponse { token, user }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(creds): Json<Credentials>,
) -> ApiResult<Json<AuthResponse>> {
    let (user, hash) = state
        .user_credentials(&creds.email)
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;
    if !finviz_auth::verify_password(&creds.password, &hash) {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }
    let token = issue(&state, &user)?;
    Ok(Json(AuthResponse { token, user }))
}

pub async fn me(AuthUser(user): AuthUser) -> Json<User> {
    Json(user)
}

pub async fn refresh(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<AuthResponse>> {
    let token = issue(&state, &user)?;
    Ok(Json(AuthResponse { token, user }))
}
