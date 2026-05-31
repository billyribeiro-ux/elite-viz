//! `/api/v1/{futures,forex,crypto}` — deterministic, synthetic market boards.
//!
//! Each endpoint returns a `Vec<MarketAsset>` generated in
//! `finviz_core::boards`. The optional `?group=` query parameter filters rows
//! to a single bucket label (case-insensitive), e.g. `?group=Metals` on the
//! futures board or `?group=Major` on forex. Prices are illustrative only.

use axum::extract::{Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::MarketAsset;
use serde::Deserialize;

/// Optional board filters.
#[derive(Debug, Default, Deserialize)]
pub struct BoardQuery {
    /// Restrict to a single `group` bucket (case-insensitive).
    pub group: Option<String>,
}

/// Apply the optional case-insensitive `group` filter, preserving row order.
fn filtered(rows: Vec<MarketAsset>, query: &BoardQuery) -> Vec<MarketAsset> {
    match query.group.as_deref() {
        Some(g) if !g.is_empty() => {
            let g = g.to_ascii_lowercase();
            rows.into_iter()
                .filter(|r| r.group.to_ascii_lowercase() == g)
                .collect()
        }
        _ => rows,
    }
}

/// `GET /api/v1/futures` — the synthetic futures board.
pub async fn futures(
    State(state): State<AppState>,
    Query(query): Query<BoardQuery>,
) -> Json<Vec<MarketAsset>> {
    Json(filtered(state.futures(), &query))
}

/// `GET /api/v1/forex` — the synthetic forex board.
pub async fn forex(
    State(state): State<AppState>,
    Query(query): Query<BoardQuery>,
) -> Json<Vec<MarketAsset>> {
    Json(filtered(state.forex(), &query))
}

/// `GET /api/v1/crypto` — the synthetic crypto board.
pub async fn crypto(
    State(state): State<AppState>,
    Query(query): Query<BoardQuery>,
) -> Json<Vec<MarketAsset>> {
    Json(filtered(state.crypto(), &query))
}
