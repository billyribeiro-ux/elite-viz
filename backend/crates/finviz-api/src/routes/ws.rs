//! `/ws/quotes` — streams periodic quote ticks for a set of symbols.
//!
//! Connect with `?symbols=AAPL,MSFT,NVDA`. The server pushes a JSON array of
//! [`QuoteTick`]s roughly once per second until the client disconnects.

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use finviz_core::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Comma-separated symbols. Defaults to a small sample set.
    symbols: Option<String>,
}

pub async fn quotes(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
) -> Response {
    let mut symbols: Vec<String> = q
        .symbols
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    if symbols.is_empty() {
        symbols = vec!["AAPL".into(), "MSFT".into(), "NVDA".into()];
    }

    ws.on_upgrade(move |socket| stream(socket, state, symbols))
}

async fn stream(mut socket: WebSocket, state: AppState, symbols: Vec<String>) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let ticks: Vec<_> = symbols.iter().filter_map(|s| state.tick(s)).collect();
                let payload = serde_json::to_string(&ticks).unwrap_or_else(|_| "[]".into());
                if socket.send(Message::Text(payload.into())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScreenerWsQuery {
    /// Screener filter expression. Empty = match all.
    query: Option<String>,
    /// Max rows per update (top movers by change %).
    limit: Option<usize>,
}

/// `/ws/screener-updates` — re-evaluates a screener query against live-jittered
/// prices roughly every 2s and pushes `{ total, matched, rows }`.
pub async fn screener_updates(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<ScreenerWsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| stream_screener(socket, state, q))
}

async fn stream_screener(mut socket: WebSocket, state: AppState, q: ScreenerWsQuery) {
    let query = q.query.unwrap_or_default();
    let limit = q.limit.unwrap_or(25).clamp(1, 200);

    let expr = if query.trim().is_empty() {
        None
    } else {
        match finviz_screener::parse(&query) {
            Ok(e) => Some(e),
            Err(e) => {
                let msg = serde_json::json!({ "error": e.to_string() }).to_string();
                let _ = socket.send(Message::Text(msg.into())).await;
                return;
            }
        }
    };

    let mut interval = tokio::time::interval(Duration::from_millis(2000));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Apply live jitter so each update reflects price movement.
                let mut rows = state.screener_rows();
                for row in rows.iter_mut() {
                    if let Some(t) = state.tick(&row.symbol) {
                        row.price = t.price;
                        row.change = t.change;
                        row.change_pct = t.change_pct;
                    }
                }
                let total = rows.len();
                let mut matched: Vec<_> = match &expr {
                    Some(e) => rows.into_iter().filter(|r| finviz_screener::evaluate(e, r)).collect(),
                    None => rows,
                };
                let matched_count = matched.len();
                matched.sort_by(|a, b| {
                    b.change_pct
                        .partial_cmp(&a.change_pct)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                matched.truncate(limit);
                let payload = serde_json::json!({
                    "total": total,
                    "matched": matched_count,
                    "rows": matched,
                })
                .to_string();
                if socket.send(Message::Text(payload.into())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }
}
