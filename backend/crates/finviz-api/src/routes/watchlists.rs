//! `/api/v1/watchlists/*` — CRUD over user watchlists.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use finviz_core::AppState;
use finviz_types::Watchlist;
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

pub async fn list(State(state): State<AppState>) -> Json<Vec<Watchlist>> {
    Json(state.watchlists())
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Watchlist>> {
    state
        .watchlist(&id)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("no watchlist `{id}`")))
}

#[derive(Debug, Deserialize)]
pub struct CreateWatchlist {
    pub name: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateWatchlist>,
) -> ApiResult<(StatusCode, Json<Watchlist>)> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("watchlist name is required".into()));
    }
    let wl = state.create_watchlist(body.name, body.symbols);
    Ok((StatusCode::CREATED, Json(wl)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWatchlist {
    pub name: Option<String>,
    pub symbols: Option<Vec<String>>,
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateWatchlist>,
) -> ApiResult<Json<Watchlist>> {
    state
        .update_watchlist(&id, body.name, body.symbols)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("no watchlist `{id}`")))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    if state.delete_watchlist(&id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("no watchlist `{id}`")))
    }
}
