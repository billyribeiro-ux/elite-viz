//! Live market-data providers.
//!
//! A [`ProviderConfig`] selects one of several upstreams (Finnhub, Polygon, or
//! a generic HTTP/webhook endpoint). The shared [`reqwest::Client`] is created
//! once. Quotes are normalized into the platform's [`Quote`] type; callers fall
//! back to the seeded dataset when a live fetch fails.

use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use finviz_types::{ProviderConfig, ProviderKind, Quote};
use serde_json::Value;

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent("elite-viz/0.1")
            .build()
            .expect("failed to build HTTP client")
    })
}

/// Fetch a single live quote for `symbol` using the configured provider.
pub async fn fetch_quote(cfg: &ProviderConfig, symbol: &str) -> Result<Quote> {
    let symbol = symbol.to_ascii_uppercase();
    let key = cfg.api_key.as_deref().unwrap_or_default();
    match cfg.kind {
        ProviderKind::Mock => bail!("mock provider has no live endpoint"),
        ProviderKind::Finnhub => finnhub(cfg, &symbol, key).await,
        ProviderKind::Polygon => polygon(cfg, &symbol, key).await,
        ProviderKind::Generic => generic(cfg, &symbol, key).await,
    }
}

/// Verify the provider is reachable and credentials work (probes `AAPL`).
pub async fn health(cfg: &ProviderConfig) -> Result<()> {
    if cfg.kind == ProviderKind::Mock {
        return Ok(());
    }
    if matches!(cfg.kind, ProviderKind::Finnhub | ProviderKind::Polygon)
        && cfg.api_key.as_deref().unwrap_or_default().is_empty()
    {
        bail!("an API key is required for this provider");
    }
    if cfg.kind == ProviderKind::Generic && cfg.base_url.as_deref().unwrap_or_default().is_empty() {
        bail!("a base URL is required for the generic provider");
    }
    fetch_quote(cfg, "AAPL").await.map(|_| ())
}

async fn finnhub(cfg: &ProviderConfig, symbol: &str, key: &str) -> Result<Quote> {
    let base = cfg
        .base_url
        .clone()
        .unwrap_or_else(|| "https://finnhub.io/api/v1".to_string());
    let url = format!(
        "{}/quote?symbol={symbol}&token={key}",
        base.trim_end_matches('/')
    );
    let v: Value = get_json(&url).await?;
    let price = num(&v, "c");
    let prev = num(&v, "pc");
    Ok(Quote {
        symbol: symbol.to_string(),
        price,
        change: opt_num(&v, "d").unwrap_or(price - prev),
        change_pct: opt_num(&v, "dp").unwrap_or_else(|| pct(price, prev)),
        volume: 0,
        prev_close: prev,
        day_high: opt_num(&v, "h").unwrap_or(price),
        day_low: opt_num(&v, "l").unwrap_or(price),
        ts: now(),
    })
}

async fn polygon(cfg: &ProviderConfig, symbol: &str, key: &str) -> Result<Quote> {
    let base = cfg
        .base_url
        .clone()
        .unwrap_or_else(|| "https://api.polygon.io".to_string());
    let url = format!(
        "{}/v2/snapshot/locale/us/markets/stocks/tickers/{symbol}?apiKey={key}",
        base.trim_end_matches('/')
    );
    let v: Value = get_json(&url).await?;
    let t = v.get("ticker").unwrap_or(&Value::Null);
    let day = t.get("day").unwrap_or(&Value::Null);
    let prev_day = t.get("prevDay").unwrap_or(&Value::Null);
    let last = t
        .get("lastTrade")
        .and_then(|lt| lt.get("p"))
        .and_then(Value::as_f64)
        .or_else(|| day.get("c").and_then(Value::as_f64))
        .ok_or_else(|| anyhow!("polygon: no price in snapshot"))?;
    let prev = prev_day.get("c").and_then(Value::as_f64).unwrap_or(last);
    Ok(Quote {
        symbol: symbol.to_string(),
        price: last,
        change: t
            .get("todaysChange")
            .and_then(Value::as_f64)
            .unwrap_or(last - prev),
        change_pct: t
            .get("todaysChangePerc")
            .and_then(Value::as_f64)
            .unwrap_or_else(|| pct(last, prev)),
        volume: day.get("v").and_then(Value::as_f64).unwrap_or(0.0) as i64,
        prev_close: prev,
        day_high: day.get("h").and_then(Value::as_f64).unwrap_or(last),
        day_low: day.get("l").and_then(Value::as_f64).unwrap_or(last),
        ts: now(),
    })
}

/// Generic provider: `GET {base_url}?symbol=SYM` with optional bearer auth.
/// Accepts a flexible JSON body and maps common field aliases.
async fn generic(cfg: &ProviderConfig, symbol: &str, key: &str) -> Result<Quote> {
    let base = cfg
        .base_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("generic provider requires a base URL"))?;
    let sep = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{sep}symbol={symbol}");

    let mut req = client().get(&url);
    if !key.is_empty() {
        req = req.bearer_auth(key);
    }
    let resp = req.send().await.context("request failed")?;
    if !resp.status().is_success() {
        bail!("provider returned HTTP {}", resp.status());
    }
    let v: Value = resp.json().await.context("invalid JSON from provider")?;

    let price = first_num(&v, &["price", "c", "last", "p", "close"])
        .ok_or_else(|| anyhow!("no recognizable price field in response"))?;
    let prev = first_num(&v, &["prev_close", "pc", "previousClose", "prevClose"]).unwrap_or(price);
    Ok(Quote {
        symbol: symbol.to_string(),
        price,
        change: first_num(&v, &["change", "d"]).unwrap_or(price - prev),
        change_pct: first_num(&v, &["change_pct", "dp", "changePercent"])
            .unwrap_or_else(|| pct(price, prev)),
        volume: first_num(&v, &["volume", "v"]).unwrap_or(0.0) as i64,
        prev_close: prev,
        day_high: first_num(&v, &["day_high", "h", "high"]).unwrap_or(price),
        day_low: first_num(&v, &["day_low", "l", "low"]).unwrap_or(price),
        ts: now(),
    })
}

async fn get_json(url: &str) -> Result<Value> {
    let resp = client().get(url).send().await.context("request failed")?;
    if !resp.status().is_success() {
        bail!("provider returned HTTP {}", resp.status());
    }
    resp.json().await.context("invalid JSON from provider")
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}
fn opt_num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}
fn first_num(v: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|k| v.get(*k).and_then(Value::as_f64))
}
fn pct(price: f64, prev: f64) -> f64 {
    if prev != 0.0 {
        (price - prev) / prev * 100.0
    } else {
        0.0
    }
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
