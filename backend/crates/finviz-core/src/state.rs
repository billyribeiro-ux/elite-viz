//! Shared application state: an in-memory market dataset.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use finviz_types::{
    Bar, Fundamentals, Instrument, Interval, Position, Quote, QuoteTick, ScreenerRow, Watchlist,
};

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
    // Mutable, user-owned collections (persisted to Postgres in a later phase).
    watchlists: RwLock<HashMap<String, Watchlist>>,
    positions: RwLock<HashMap<String, Position>>,
    next_id: AtomicU64,
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

        let mut watchlists = HashMap::new();
        watchlists.insert(
            "default".to_string(),
            Watchlist {
                id: "default".to_string(),
                name: "My Watchlist".to_string(),
                symbols: vec!["AAPL".into(), "MSFT".into(), "NVDA".into(), "TSLA".into()],
            },
        );

        let mut positions = HashMap::new();
        for (symbol, quantity, avg_price) in [("AAPL", 50.0, 180.0), ("NVDA", 100.0, 95.0)] {
            positions.insert(
                symbol.to_string(),
                Position {
                    symbol: symbol.to_string(),
                    quantity,
                    avg_price,
                },
            );
        }

        Self {
            inner: Arc::new(Data {
                instruments,
                quotes,
                fundamentals,
                watchlists: RwLock::new(watchlists),
                positions: RwLock::new(positions),
                next_id: AtomicU64::new(1),
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

    /// `true` if the symbol exists in the dataset.
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.inner.quotes.contains_key(&symbol.to_ascii_uppercase())
    }

    // ---- Watchlists --------------------------------------------------------

    pub fn watchlists(&self) -> Vec<Watchlist> {
        let mut v: Vec<Watchlist> = self
            .inner
            .watchlists
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    pub fn watchlist(&self, id: &str) -> Option<Watchlist> {
        self.inner.watchlists.read().unwrap().get(id).cloned()
    }

    pub fn create_watchlist(&self, name: String, symbols: Vec<String>) -> Watchlist {
        let id = format!("wl-{}", self.inner.next_id.fetch_add(1, Ordering::Relaxed));
        let wl = Watchlist {
            id: id.clone(),
            name,
            symbols: normalize_symbols(symbols),
        };
        self.inner
            .watchlists
            .write()
            .unwrap()
            .insert(id, wl.clone());
        wl
    }

    pub fn update_watchlist(
        &self,
        id: &str,
        name: Option<String>,
        symbols: Option<Vec<String>>,
    ) -> Option<Watchlist> {
        let mut guard = self.inner.watchlists.write().unwrap();
        let wl = guard.get_mut(id)?;
        if let Some(name) = name {
            wl.name = name;
        }
        if let Some(symbols) = symbols {
            wl.symbols = normalize_symbols(symbols);
        }
        Some(wl.clone())
    }

    pub fn delete_watchlist(&self, id: &str) -> bool {
        self.inner.watchlists.write().unwrap().remove(id).is_some()
    }

    // ---- Portfolio ---------------------------------------------------------

    pub fn positions(&self) -> Vec<Position> {
        let mut v: Vec<Position> = self
            .inner
            .positions
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect();
        v.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        v
    }

    /// Insert or replace a position, keyed by symbol.
    pub fn upsert_position(&self, position: Position) -> Position {
        let position = Position {
            symbol: position.symbol.to_ascii_uppercase(),
            ..position
        };
        self.inner
            .positions
            .write()
            .unwrap()
            .insert(position.symbol.clone(), position.clone());
        position
    }

    pub fn delete_position(&self, symbol: &str) -> bool {
        self.inner
            .positions
            .write()
            .unwrap()
            .remove(&symbol.to_ascii_uppercase())
            .is_some()
    }

    // ---- Realtime ----------------------------------------------------------

    /// Produce a lightly-jittered live tick for a symbol, anchored to its seed
    /// quote. Stateless: each call re-jitters around the base price.
    pub fn tick(&self, symbol: &str) -> Option<QuoteTick> {
        let base = self.quote(symbol)?;
        let now = unix_now();
        // Deterministic-ish jitter from the wall clock, +/-0.25%.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let unit = (nanos as f64 / u32::MAX as f64) - 0.5;
        let price = (base.price * (1.0 + unit * 0.005)).max(0.01);
        let change = price - base.prev_close;
        let change_pct = if base.prev_close != 0.0 {
            change / base.prev_close * 100.0
        } else {
            0.0
        };
        Some(QuoteTick {
            symbol: base.symbol,
            price: (price * 100.0).round() / 100.0,
            change: (change * 100.0).round() / 100.0,
            change_pct: (change_pct * 100.0).round() / 100.0,
            ts: now,
        })
    }
}

fn normalize_symbols(symbols: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    symbols
        .into_iter()
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .collect()
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
            let open = if i == 0 { close } else { closes[i - 1] };
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
