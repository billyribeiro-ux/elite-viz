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
