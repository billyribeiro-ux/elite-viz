//! API v1 router assembly.

pub mod alerts;
pub mod auth;
pub mod backtest;
pub mod boards;
pub mod etf;
pub mod export;
pub mod groups;
pub mod indicators;
pub mod market_data;
pub mod news;
pub mod options;
pub mod patterns;
pub mod portfolio;
pub mod saved_screens;
pub mod screener;
pub mod settings;
pub mod watchlists;
pub mod ws;

use axum::routing::{delete, get, post, put};
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
        .route("/market-data/insider/{symbol}", get(news::insider))
        .route("/market-data/ratings/{symbol}", get(news::ratings))
        // news
        .route("/news", get(news::list))
        // screener
        .route("/screener/run", post(screener::run))
        .route("/screener/fields", get(screener::fields))
        .route("/screener/presets", get(screener::presets))
        // saved screens
        .route(
            "/screener/saved",
            get(saved_screens::list).post(saved_screens::create),
        )
        .route(
            "/screener/saved/{id}",
            put(saved_screens::update).delete(saved_screens::delete),
        )
        // indicators
        .route("/indicators/sma/{symbol}", get(indicators::sma))
        .route("/indicators/rsi/{symbol}", get(indicators::rsi))
        .route("/indicators/ema/{symbol}", get(indicators::ema))
        .route("/indicators/macd/{symbol}", get(indicators::macd))
        .route("/indicators/bbands/{symbol}", get(indicators::bbands))
        .route("/indicators/atr/{symbol}", get(indicators::atr))
        // backtest
        .route("/backtest", post(backtest::run))
        .route("/backtest/rules", get(backtest::rules))
        // groups
        .route("/groups", get(groups::list))
        // options chain
        .route("/options/{symbol}", get(options::chain))
        // chart patterns
        .route("/patterns/{symbol}", get(patterns::detect))
        // ETF analysis
        .route("/etf", get(etf::list))
        .route("/etf/{symbol}", get(etf::get))
        // market boards (futures / forex / crypto)
        .route("/futures", get(boards::futures))
        .route("/forex", get(boards::forex))
        .route("/crypto", get(boards::crypto))
        // CSV export
        .route("/export/screener", post(export::screener))
        .route("/export/groups", get(export::groups))
        .route("/export/portfolio", get(export::portfolio))
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
        .route("/portfolio/positions/{symbol}", delete(portfolio::delete))
        .route("/portfolio/summary", get(portfolio::summary))
        // alerts
        .route("/alerts", get(alerts::list).post(alerts::create))
        .route("/alerts/{id}", delete(alerts::delete))
        .route("/alerts/check", get(alerts::check))
        // settings
        .route(
            "/settings/provider",
            get(settings::get).put(settings::update),
        )
        .route("/settings/provider/test", post(settings::test))
        // auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .route("/auth/refresh", post(auth::refresh))
}
