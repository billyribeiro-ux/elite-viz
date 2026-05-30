//! `/api/v1/portfolio/*` — positions and valuation.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use finviz_core::AppState;
use finviz_types::{PortfolioSummary, Position, PositionValue};
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

pub async fn list_positions(State(state): State<AppState>) -> Json<Vec<Position>> {
    Json(state.positions())
}

pub async fn summary(State(state): State<AppState>) -> Json<PortfolioSummary> {
    let mut positions = Vec::new();
    let mut market_value = 0.0;
    let mut cost_basis = 0.0;

    for p in state.positions() {
        let last_price = state
            .quote(&p.symbol)
            .map(|q| q.price)
            .unwrap_or(p.avg_price);
        let mv = last_price * p.quantity;
        let cb = p.avg_price * p.quantity;
        let pnl = mv - cb;
        market_value += mv;
        cost_basis += cb;
        positions.push(PositionValue {
            symbol: p.symbol,
            quantity: p.quantity,
            avg_price: p.avg_price,
            last_price,
            market_value: round2(mv),
            cost_basis: round2(cb),
            unrealized_pnl: round2(pnl),
            unrealized_pnl_pct: if cb != 0.0 {
                round2(pnl / cb * 100.0)
            } else {
                0.0
            },
        });
    }

    let pnl = market_value - cost_basis;
    Json(PortfolioSummary {
        positions,
        market_value: round2(market_value),
        cost_basis: round2(cost_basis),
        unrealized_pnl: round2(pnl),
        unrealized_pnl_pct: if cost_basis != 0.0 {
            round2(pnl / cost_basis * 100.0)
        } else {
            0.0
        },
    })
}

#[derive(Debug, Deserialize)]
pub struct UpsertPosition {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(body): Json<UpsertPosition>,
) -> ApiResult<Json<Position>> {
    if !state.has_symbol(&body.symbol) {
        return Err(AppError::BadRequest(format!(
            "unknown symbol `{}`",
            body.symbol
        )));
    }
    if body.quantity <= 0.0 || body.avg_price <= 0.0 {
        return Err(AppError::BadRequest(
            "quantity and avg_price must be positive".into(),
        ));
    }
    Ok(Json(state.upsert_position(Position {
        symbol: body.symbol,
        quantity: body.quantity,
        avg_price: body.avg_price,
    })))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> ApiResult<StatusCode> {
    if state.delete_position(&symbol) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("no position for `{symbol}`")))
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
