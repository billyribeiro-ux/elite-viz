//! `/api/v1/screener/*` — run filter queries, list fields, list presets.

use std::cmp::Ordering;

use axum::extract::State;
use axum::Json;
use finviz_core::AppState;
use finviz_screener::value::{canonical_field, Row, Value};
use finviz_types::ScreenerRow;
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Deserialize)]
pub struct ScreenRequest {
    /// Filter DSL, e.g. `price > 100 and pe < 30`. Empty = match all.
    #[serde(default)]
    pub query: String,
    /// Field to sort by (defaults to market cap).
    pub sort: Option<String>,
    #[serde(default)]
    pub order: SortOrder,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ScreenResponse {
    pub query: String,
    /// Rows in the dataset before filtering.
    pub total: usize,
    /// Rows matching the filter (before pagination).
    pub matched: usize,
    pub rows: Vec<ScreenerRow>,
}

pub async fn run(
    State(state): State<AppState>,
    Json(req): Json<ScreenRequest>,
) -> ApiResult<Json<ScreenResponse>> {
    let all = state.screener_rows();
    let total = all.len();

    let mut matched: Vec<ScreenerRow> = if req.query.trim().is_empty() {
        all
    } else {
        let expr =
            finviz_screener::parse(&req.query).map_err(|e| AppError::BadRequest(e.to_string()))?;
        all.into_iter()
            .filter(|row| finviz_screener::evaluate(&expr, row))
            .collect()
    };

    let sort_field = req.sort.as_deref().unwrap_or("market_cap");
    let key = canonical_field(sort_field);
    if key.is_empty() {
        return Err(AppError::BadRequest(format!(
            "unknown sort field `{sort_field}`"
        )));
    }
    matched.sort_by(|a, b| {
        let ord = compare_values(&a.field(key), &b.field(key));
        match req.order {
            SortOrder::Asc => ord,
            SortOrder::Desc => ord.reverse(),
        }
    });

    let matched_count = matched.len();
    let offset = req.offset.unwrap_or(0);
    let limit = req.limit.unwrap_or(100).min(1000);
    let rows = matched.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ScreenResponse {
        query: req.query,
        total,
        matched: matched_count,
        rows,
    }))
}

fn compare_values(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (Value::Str(x), Value::Str(y)) => x.to_ascii_lowercase().cmp(&y.to_ascii_lowercase()),
        // Nulls sort last under ascending order.
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Greater,
        (_, Value::Null) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

#[derive(Debug, Serialize)]
pub struct FieldInfo {
    pub name: &'static str,
    pub kind: &'static str,
}

pub async fn fields() -> Json<Vec<FieldInfo>> {
    Json(
        finviz_screener::known_fields()
            .iter()
            .map(|&(name, kind)| FieldInfo { name, kind })
            .collect(),
    )
}

#[derive(Debug, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub label: &'static str,
    pub query: &'static str,
}

pub async fn presets() -> Json<Vec<Preset>> {
    Json(vec![
        Preset {
            id: "mega-cap-tech",
            label: "Mega-cap Technology",
            query: "sector = \"Technology\" and market_cap > 1e12",
        },
        Preset {
            id: "value",
            label: "Value (low P/E, pays a dividend)",
            query: "pe < 20 and dividend_yield > 2",
        },
        Preset {
            id: "high-beta-movers",
            label: "High-beta movers",
            query: "beta > 1.3 and change_pct > 1",
        },
        Preset {
            id: "large-cap-gainers",
            label: "Large-cap gainers",
            query: "market_cap > 5e11 and change_pct > 0",
        },
    ])
}
