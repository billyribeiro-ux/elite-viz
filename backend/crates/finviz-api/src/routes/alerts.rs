//! `/api/v1/alerts/*` — price/metric alerts built on the screener DSL.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use finviz_core::AppState;
use finviz_types::Alert;
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};

pub async fn list(State(state): State<AppState>) -> Json<Vec<Alert>> {
    Json(state.alerts())
}

#[derive(Debug, Deserialize)]
pub struct CreateAlert {
    pub symbol: String,
    /// A screener expression evaluated against the symbol, e.g. `price > 250`.
    pub query: String,
    #[serde(default)]
    pub note: String,
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateAlert>,
) -> ApiResult<(StatusCode, Json<Alert>)> {
    if !state.has_symbol(&body.symbol) {
        return Err(AppError::BadRequest(format!(
            "unknown symbol `{}`",
            body.symbol
        )));
    }
    finviz_screener::parse(&body.query).map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(state.create_alert(body.symbol, body.query, body.note)),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    if state.delete_alert(&id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("no alert `{id}`")))
    }
}

#[derive(Debug, Serialize)]
pub struct AlertStatus {
    #[serde(flatten)]
    pub alert: Alert,
    pub triggered: bool,
}

/// Evaluate every alert against current market data.
pub async fn check(State(state): State<AppState>) -> Json<Vec<AlertStatus>> {
    let statuses = state
        .alerts()
        .into_iter()
        .map(|alert| {
            let triggered = finviz_screener::parse(&alert.query)
                .ok()
                .zip(state.screener_row(&alert.symbol))
                .map(|(expr, row)| finviz_screener::evaluate(&expr, &row))
                .unwrap_or(false);
            AlertStatus { alert, triggered }
        })
        .collect();
    Json(statuses)
}
