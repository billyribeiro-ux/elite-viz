//! Shared application state: an in-memory market dataset.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use finviz_types::{Bar, Fundamentals, Instrument, Interval, Quote, ScreenerRow};

use crate::seed;

/// Cheap-to-clone handle to the shared dataset (`Arc` inside).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Data>,
}

struct Data {
    instruments: Vec<Instrument>,
    quotes: HashMap<String, Quote>,
    fundamentals: HashMap<String, Fundamentals>,
}

impl AppState {
    /// Build state from the bundled synthetic seed dataset.
    pub fn seeded() -> Self {
        let now = unix_now();
        let mut instruments = Vec::new();
        let mut quotes = HashMap::new();
        let mut fundamentals = HashMap::new();

        for row in seed::dataset(now) {
            quotes.insert(row.instrument.symbol.clone(), row.quote);
            fundamentals.insert(row.instrument.symbol.clone(), row.fundamentals);
            instruments.push(row.instrument);
        }

        Self {
            inner: Arc::new(Data {
                instruments,
                quotes,
                fundamentals,
            }),
        }
    }

    pub fn instruments(&self) -> &[Instrument] {
        &self.inner.instruments
    }

    pub fn quote(&self, symbol: &str) -> Option<Quote> {
        self.inner.quotes.get(&symbol.to_ascii_uppercase()).cloned()
    }

    pub fn fundamentals(&self, symbol: &str) -> Option<Fundamentals> {
        self.inner
            .fundamentals
            .get(&symbol.to_ascii_uppercase())
            .cloned()
    }

    /// Merge instruments + quotes + fundamentals into flat screener rows.
    pub fn screener_rows(&self) -> Vec<ScreenerRow> {
        self.inner
            .instruments
            .iter()
            .filter_map(|inst| {
                let q = self.inner.quotes.get(&inst.symbol)?;
                let f = self.inner.fundamentals.get(&inst.symbol)?;
                Some(ScreenerRow {
                    symbol: inst.symbol.clone(),
                    name: inst.name.clone(),
                    sector: inst.sector.clone(),
                    industry: inst.industry.clone(),
                    exchange: inst.exchange.clone(),
                    price: q.price,
                    change: q.change,
                    change_pct: q.change_pct,
                    volume: q.volume,
                    market_cap: f.market_cap,
                    pe: f.pe,
                    eps: f.eps,
                    dividend_yield: f.dividend_yield,
                    beta: f.beta,
                })
            })
            .collect()
    }

    /// Deterministically synthesize OHLCV history ending at the latest quote.
    pub fn bars(&self, symbol: &str, interval: Interval, limit: usize) -> Option<Vec<Bar>> {
        let quote = self.quote(symbol)?;
        Some(synth_bars(symbol, &quote, interval, limit.clamp(1, 1000)))
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Generate a reproducible random-walk series anchored to the current price.
/// Same inputs always yield the same bars (seeded by symbol + interval).
fn synth_bars(symbol: &str, quote: &Quote, interval: Interval, count: usize) -> Vec<Bar> {
    let mut rng = Lcg::seeded(symbol, interval);
    let step = interval.seconds();
    let end_ts = quote.ts - (quote.ts % step);

    // Walk backwards from the current price, then emit oldest-first.
    let mut closes = vec![quote.price];
    for _ in 1..count {
        let prev = *closes.last().unwrap();
        let drift = (rng.next_unit() - 0.5) * 0.02; // +/-1% per bar
        closes.push((prev / (1.0 + drift)).max(0.01));
    }
    closes.reverse();

    closes
        .iter()
        .enumerate()
        .map(|(i, &close)| {
            let open = if i == 0 {
                close
            } else {
                closes[i - 1]
            };
            let spread = close * (rng.next_unit() * 0.01);
            let high = open.max(close) + spread;
            let low = (open.min(close) - spread).max(0.01);
            let vol_jitter = 0.5 + rng.next_unit();
            Bar {
                ts: end_ts - (count as i64 - 1 - i as i64) * step,
                open,
                high,
                low,
                close,
                volume: ((quote.volume as f64 / count as f64) * vol_jitter) as i64,
            }
        })
        .collect()
}

/// A tiny deterministic linear-congruential generator (not for crypto).
struct Lcg {
    state: u64,
}

impl Lcg {
    fn seeded(symbol: &str, interval: Interval) -> Self {
        let mut state = 0xcbf29ce484222325u64; // FNV offset basis
        for b in symbol.bytes() {
            state = (state ^ b as u64).wrapping_mul(0x100000001b3);
        }
        state ^= interval.seconds() as u64;
        Self { state: state | 1 }
    }

    fn next_unit(&mut self) -> f64 {
        // LCG constants from Numerical Recipes.
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 11) as f64) / ((1u64 << 53) as f64)
    }
}
