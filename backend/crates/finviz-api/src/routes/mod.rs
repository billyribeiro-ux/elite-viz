//! API v1 router assembly.

pub mod indicators;
pub mod market_data;
pub mod screener;

use axum::routing::{get, post};
use axum::Router;
use finviz_core::AppState;

/// Routes mounted under `/api/v1`.
pub fn api_router() -> Router<AppState> {
    Router::new()
        // market-data
        .route("/market-data/instruments", get(market_data::instruments))
        .route("/market-data/quote/{symbol}", get(market_data::quote))
        .route(
            "/market-data/fundamentals/{symbol}",
            get(market_data::fundamentals),
        )
        .route("/market-data/bars/{symbol}", get(market_data::bars))
        // screener
        .route("/screener/run", post(screener::run))
        .route("/screener/fields", get(screener::fields))
        .route("/screener/presets", get(screener::presets))
        // indicators
        .route("/indicators/sma/{symbol}", get(indicators::sma))
        .route("/indicators/rsi/{symbol}", get(indicators::rsi))
}
