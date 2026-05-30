//! API v1 router assembly.

pub mod indicators;
pub mod market_data;
pub mod portfolio;
pub mod screener;
pub mod watchlists;
pub mod ws;

use axum::routing::{delete, get, post};
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
        // watchlists
        .route(
            "/watchlists",
            get(watchlists::list).post(watchlists::create),
        )
        .route(
            "/watchlists/{id}",
            get(watchlists::get)
                .put(watchlists::update)
                .delete(watchlists::delete),
        )
        // portfolio
        .route(
            "/portfolio/positions",
            get(portfolio::list_positions).post(portfolio::upsert),
        )
        .route(
            "/portfolio/positions/{symbol}",
            delete(portfolio::delete),
        )
        .route("/portfolio/summary", get(portfolio::summary))
}
