//! `/api/v1/etf` — deterministic, synthetic ETF profiles + holdings.
//!
//! The seed universe is single stocks, so a small set of synthetic ETFs (SPY,
//! QQQ, DIA, IWM, XLK) is designated in `finviz_core::derivatives`; holdings are
//! drawn from the seed instruments and weighted by market cap.

use axum::extract::{Path, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::EtfProfile;

use crate::error::{ApiResult, AppError};

/// `GET /api/v1/etf` — all designated synthetic ETF profiles, ascending by
/// symbol.
pub async fn list(State(state): State<AppState>) -> Json<Vec<EtfProfile>> {
    Json(state.etfs())
}

/// `GET /api/v1/etf/{symbol}` — one ETF profile. 404 if not a designated ETF.
pub async fn get(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<EtfProfile>> {
    match state.etf_profile(&symbol) {
        Some(profile) => Ok(Json(profile)),
        None => Err(AppError::NotFound(format!("`{symbol}` is not an ETF"))),
    }
}
