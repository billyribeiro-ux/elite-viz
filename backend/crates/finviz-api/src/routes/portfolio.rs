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
    // Resolve each position's mark to the latest quote, falling back to its
    // cost basis when no quote exists, then aggregate.
    let priced = state.positions().into_iter().map(|p| {
        let last_price = state.quote(&p.symbol).map_or(p.avg_price, |q| q.price);
        (p, last_price)
    });
    Json(summarize(priced))
}

/// Pure portfolio valuation: fold `(position, last_price)` pairs into a summary
/// with per-position and aggregate market value, cost basis, and unrealized
/// P&L. Percentages guard against a zero cost basis.
fn summarize(priced: impl Iterator<Item = (Position, f64)>) -> PortfolioSummary {
    let mut positions = Vec::new();
    let mut market_value = 0.0;
    let mut cost_basis = 0.0;

    for (p, last_price) in priced {
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
            unrealized_pnl_pct: pct(pnl, cb),
        });
    }

    let pnl = market_value - cost_basis;
    PortfolioSummary {
        positions,
        market_value: round2(market_value),
        cost_basis: round2(cost_basis),
        unrealized_pnl: round2(pnl),
        unrealized_pnl_pct: pct(pnl, cost_basis),
    }
}

/// `pnl` as a percentage of `cost_basis`, or 0 when the basis is zero.
fn pct(pnl: f64, cost_basis: f64) -> f64 {
    if cost_basis == 0.0 {
        0.0
    } else {
        round2(pnl / cost_basis * 100.0)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(symbol: &str, quantity: f64, avg_price: f64) -> Position {
        Position {
            symbol: symbol.into(),
            quantity,
            avg_price,
        }
    }

    #[test]
    fn summarizes_gain_and_loss_with_correct_aggregates() {
        // 10 sh @ 100 now 110  => +100  (+10%)
        // 5 sh  @ 200 now 150  => -250  (-25%)
        let summary = summarize(
            [
                (pos("AAA", 10.0, 100.0), 110.0),
                (pos("BBB", 5.0, 200.0), 150.0),
            ]
            .into_iter(),
        );

        assert_eq!(summary.positions[0].unrealized_pnl, 100.0);
        assert_eq!(summary.positions[0].unrealized_pnl_pct, 10.0);
        assert_eq!(summary.positions[1].unrealized_pnl, -250.0);
        assert_eq!(summary.positions[1].unrealized_pnl_pct, -25.0);

        assert_eq!(summary.market_value, 110.0 * 10.0 + 150.0 * 5.0);
        assert_eq!(summary.cost_basis, 100.0 * 10.0 + 200.0 * 5.0);
        assert_eq!(summary.unrealized_pnl, -150.0);
        // -150 / 2000 * 100 = -7.5%
        assert_eq!(summary.unrealized_pnl_pct, -7.5);
    }

    #[test]
    fn empty_portfolio_has_zero_percentage_not_nan() {
        let summary = summarize(std::iter::empty());
        assert_eq!(summary.cost_basis, 0.0);
        assert_eq!(summary.unrealized_pnl_pct, 0.0);
        assert!(summary.positions.is_empty());
    }
}
