//! `/api/v1/export/*` — CSV downloads of screener results, group
//! aggregations, and portfolio positions.
//!
//! The CSV serialization is a tiny RFC-4180-style writer (no external crate):
//! a field is quoted (and embedded quotes doubled) only when it contains a
//! comma, a double-quote, or a CR/LF; records are separated by CRLF.

use std::cmp::Ordering;

use axum::extract::{Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use finviz_core::AppState;
use finviz_screener::value::{canonical_field, Row, Value};
use finviz_types::ScreenerRow;
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

/// Quote a single CSV field if it contains a comma, quote, CR, or LF.
fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Join one record's fields with commas, escaping each.
fn csv_record<I, S>(fields: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fields
        .into_iter()
        .map(|f| csv_escape(f.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

/// Assemble a full CSV document (header + rows) using CRLF line endings, with
/// a trailing CRLF after the final record.
fn csv_document(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(&csv_record(header.iter().copied()));
    out.push_str("\r\n");
    for row in rows {
        out.push_str(&csv_record(row.iter().map(String::as_str)));
        out.push_str("\r\n");
    }
    out
}

/// Build a `text/csv` attachment response.
fn csv_response(filename: &str, body: String) -> Response {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
        .into_response()
}

/// Render an `f64` plainly, or empty string for an absent optional.
fn num(v: f64) -> String {
    v.to_string()
}

fn opt(v: Option<f64>) -> String {
    v.map(|n| n.to_string()).unwrap_or_default()
}

// ---- Screener export -------------------------------------------------------

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Deserialize)]
pub struct ExportScreenRequest {
    #[serde(default)]
    pub query: String,
    pub sort: Option<String>,
    #[serde(default)]
    pub order: SortOrder,
    pub limit: Option<usize>,
}

const SCREENER_HEADER: &[&str] = &[
    "symbol",
    "name",
    "sector",
    "industry",
    "country",
    "price",
    "change_pct",
    "volume",
    "market_cap",
    "pe",
    "forward_pe",
    "peg",
    "ps",
    "pb",
    "roe",
    "roa",
    "debt_equity",
    "profit_margin",
    "short_float",
    "inst_own",
    "perf_week",
    "perf_month",
    "perf_year",
    "rsi14",
];

fn screener_record(r: &ScreenerRow) -> Vec<String> {
    vec![
        r.symbol.clone(),
        r.name.clone(),
        r.sector.clone(),
        r.industry.clone(),
        r.country.clone(),
        num(r.price),
        num(r.change_pct),
        r.volume.to_string(),
        num(r.market_cap),
        opt(r.pe),
        opt(r.forward_pe),
        opt(r.peg),
        opt(r.ps),
        opt(r.pb),
        opt(r.roe),
        opt(r.roa),
        opt(r.debt_equity),
        opt(r.profit_margin),
        opt(r.short_float),
        opt(r.inst_own),
        num(r.perf_week),
        num(r.perf_month),
        num(r.perf_year),
        num(r.rsi14),
    ]
}

fn compare_values(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (Value::Str(x), Value::Str(y)) => x.to_ascii_lowercase().cmp(&y.to_ascii_lowercase()),
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Greater,
        (_, Value::Null) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

pub async fn screener(
    State(state): State<AppState>,
    Json(req): Json<ExportScreenRequest>,
) -> ApiResult<Response> {
    let all = state.screener_rows();

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

    let limit = req.limit.unwrap_or(usize::MAX).min(10_000);
    let rows: Vec<Vec<String>> = matched.iter().take(limit).map(screener_record).collect();

    let body = csv_document(SCREENER_HEADER, &rows);
    Ok(csv_response("screener.csv", body))
}

// ---- Groups export ---------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GroupsQuery {
    by: Option<String>,
}

const GROUPS_HEADER: &[&str] = &[
    "name",
    "count",
    "avg_change_pct",
    "avg_pe",
    "total_market_cap",
    "avg_perf_week",
    "avg_perf_month",
    "avg_perf_year",
];

pub async fn groups(
    State(state): State<AppState>,
    Query(q): Query<GroupsQuery>,
) -> ApiResult<Response> {
    let by = q.by.as_deref().unwrap_or("sector");
    let key = super::groups::GroupKey::parse(by)
        .ok_or_else(|| AppError::BadRequest(format!("unknown group key `{by}`")))?;
    let groups = super::groups::aggregate(&state.screener_rows(), key);

    let rows: Vec<Vec<String>> = groups
        .iter()
        .map(|g| {
            vec![
                g.name.clone(),
                g.count.to_string(),
                num(g.avg_change_pct),
                opt(g.avg_pe),
                num(g.total_market_cap),
                num(g.avg_perf_week),
                num(g.avg_perf_month),
                num(g.avg_perf_year),
            ]
        })
        .collect();

    let body = csv_document(GROUPS_HEADER, &rows);
    Ok(csv_response(&format!("groups-{by}.csv"), body))
}

// ---- Portfolio export ------------------------------------------------------

const PORTFOLIO_HEADER: &[&str] = &[
    "symbol",
    "quantity",
    "avg_price",
    "last_price",
    "market_value",
    "cost_basis",
    "unrealized_pnl",
    "unrealized_pnl_pct",
];

pub async fn portfolio(State(state): State<AppState>) -> Response {
    let summary = super::portfolio::compute_summary(&state);
    let rows: Vec<Vec<String>> = summary
        .positions
        .iter()
        .map(|p| {
            vec![
                p.symbol.clone(),
                num(p.quantity),
                num(p.avg_price),
                num(p.last_price),
                num(p.market_value),
                num(p.cost_basis),
                num(p.unrealized_pnl),
                num(p.unrealized_pnl_pct),
            ]
        })
        .collect();

    let body = csv_document(PORTFOLIO_HEADER, &rows);
    csv_response("portfolio.csv", body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_comma_quote_and_newline() {
        // Comma -> quoted.
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        // Quote -> doubled and wrapped.
        assert_eq!(csv_escape("a\"b"), "\"a\"\"b\"");
        // Newline -> quoted.
        assert_eq!(csv_escape("a\nb"), "\"a\nb\"");
        // CR -> quoted.
        assert_eq!(csv_escape("a\rb"), "\"a\rb\"");
        // Plain -> untouched.
        assert_eq!(csv_escape("plain"), "plain");
    }

    #[test]
    fn record_with_mixed_specials_round_trips_shape() {
        // One field has a comma, a quote, and a newline all at once.
        let nasty = "x,\"y\"\nz";
        let rec = csv_record([nasty, "ok"]);
        // The whole nasty field is wrapped; inner quote doubled; "ok" untouched.
        assert_eq!(rec, "\"x,\"\"y\"\"\nz\",ok");
    }

    #[test]
    fn document_has_header_then_n_rows_crlf_terminated() {
        let header = &["a", "b"];
        let rows = vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "x,y".to_string()],
        ];
        let doc = csv_document(header, &rows);
        let lines: Vec<&str> = doc.split("\r\n").collect();
        // header, row1, row2, and a trailing empty from the final CRLF.
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "a,b");
        assert_eq!(lines[1], "1,2");
        assert_eq!(lines[2], "3,\"x,y\"");
        assert_eq!(lines[3], "");
    }
}
