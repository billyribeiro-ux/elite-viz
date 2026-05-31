//! Shared application state: an in-memory market dataset.
//!
//! The mutable collections sit behind [`RwLock`]s and the accessors below use
//! `.read().unwrap()` / `.write().unwrap()`. A `PoisonError` can only arise if a
//! thread panics *while holding* one of these locks; every critical section
//! here is a short, panic-free map/clone/sort over owned data, so propagating
//! the panic (a poisoned lock means state is already suspect) is the correct,
//! standard-library-idiomatic behavior rather than silently masking corruption.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use finviz_types::{
    Alert, AnalystRating, Bar, Fundamentals, InsiderTrade, Instrument, Interval, NewsItem,
    Position, ProviderConfig, Quote, QuoteTick, SavedScreen, ScreenerRow, User, Watchlist,
};

use crate::news;
use crate::seed::{self, ScreenerExtras};

/// Internal user record including the password hash (never serialized out).
#[derive(Clone)]
struct UserRecord {
    id: String,
    email: String,
    password_hash: String,
}

/// Cheap-to-clone handle to the shared dataset (`Arc` inside).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Data>,
}

struct Data {
    instruments: Vec<Instrument>,
    quotes: HashMap<String, Quote>,
    fundamentals: HashMap<String, Fundamentals>,
    // Extended FINVIZ-style metrics (synthesized deterministically in `seed`).
    extras: HashMap<String, ScreenerExtras>,
    // Mutable, user-owned collections (persisted to Postgres in a later phase).
    watchlists: RwLock<HashMap<String, Watchlist>>,
    positions: RwLock<HashMap<String, Position>>,
    saved_screens: RwLock<HashMap<String, SavedScreen>>,
    next_id: AtomicU64,
    // Live market-data provider settings, editable at runtime via the API.
    provider: RwLock<ProviderConfig>,
    // Auth + alerts.
    users: RwLock<HashMap<String, UserRecord>>,
    alerts: RwLock<HashMap<String, Alert>>,
    jwt_secret: String,
    // Fixed anchor for synthetic news/insider/rating timestamps, captured once
    // at boot. Generated *content* never depends on the wall clock; only the
    // absolute timestamps are offset backwards from this base.
    news_base_ts: i64,
}

impl AppState {
    /// Build state from the bundled synthetic seed dataset.
    pub fn seeded() -> Self {
        let now = unix_now();
        let mut instruments = Vec::new();
        let mut quotes = HashMap::new();
        let mut fundamentals = HashMap::new();
        let mut extras = HashMap::new();

        for row in seed::dataset(now) {
            quotes.insert(row.instrument.symbol.clone(), row.quote);
            fundamentals.insert(row.instrument.symbol.clone(), row.fundamentals);
            extras.insert(row.instrument.symbol.clone(), row.extras);
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
                extras,
                watchlists: RwLock::new(watchlists),
                positions: RwLock::new(positions),
                saved_screens: RwLock::new(HashMap::new()),
                next_id: AtomicU64::new(1),
                provider: RwLock::new(ProviderConfig::default()),
                users: RwLock::new(HashMap::new()),
                alerts: RwLock::new(HashMap::new()),
                jwt_secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-change-me".to_string()),
                news_base_ts: now,
            }),
        }
    }

    /// Secret used to sign/verify JWTs.
    pub fn jwt_secret(&self) -> &[u8] {
        self.inner.jwt_secret.as_bytes()
    }

    /// Build a single merged screener row for one symbol.
    pub fn screener_row(&self, symbol: &str) -> Option<ScreenerRow> {
        let symbol = symbol.to_ascii_uppercase();
        self.screener_rows()
            .into_iter()
            .find(|r| r.symbol == symbol)
    }

    /// Current live-data provider settings.
    pub fn provider_config(&self) -> ProviderConfig {
        self.inner.provider.read().unwrap().clone()
    }

    /// Replace the provider settings. An empty incoming `api_key` preserves the
    /// existing key, so the UI can save other fields without re-entering it.
    pub fn set_provider_config(&self, mut cfg: ProviderConfig) -> ProviderConfig {
        let mut guard = self.inner.provider.write().unwrap();
        let key_missing = cfg.api_key.as_deref().is_none_or(str::is_empty);
        if key_missing {
            cfg.api_key = guard.api_key.clone();
        }
        *guard = cfg.clone();
        cfg
    }

    // ---- Users / auth ------------------------------------------------------

    /// Create a user. Returns `None` if the email is already registered.
    pub fn create_user(&self, email: &str, password_hash: String) -> Option<User> {
        let email = email.trim().to_ascii_lowercase();
        let mut guard = self.inner.users.write().unwrap();
        if guard.contains_key(&email) {
            return None;
        }
        let id = format!(
            "user-{}",
            self.inner.next_id.fetch_add(1, Ordering::Relaxed)
        );
        guard.insert(
            email.clone(),
            UserRecord {
                id: id.clone(),
                email: email.clone(),
                password_hash,
            },
        );
        Some(User { id, email })
    }

    /// Look up a user and their stored password hash by email.
    pub fn user_credentials(&self, email: &str) -> Option<(User, String)> {
        let email = email.trim().to_ascii_lowercase();
        self.inner.users.read().unwrap().get(&email).map(|r| {
            (
                User {
                    id: r.id.clone(),
                    email: r.email.clone(),
                },
                r.password_hash.clone(),
            )
        })
    }

    pub fn user_by_id(&self, id: &str) -> Option<User> {
        self.inner
            .users
            .read()
            .unwrap()
            .values()
            .find(|r| r.id == id)
            .map(|r| User {
                id: r.id.clone(),
                email: r.email.clone(),
            })
    }

    // ---- Alerts ------------------------------------------------------------

    pub fn alerts(&self) -> Vec<Alert> {
        let mut v: Vec<Alert> = self
            .inner
            .alerts
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect();
        v.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        v
    }

    pub fn create_alert(&self, symbol: String, query: String, note: String) -> Alert {
        let id = format!(
            "alert-{}",
            self.inner.next_id.fetch_add(1, Ordering::Relaxed)
        );
        let alert = Alert {
            id: id.clone(),
            symbol: symbol.to_ascii_uppercase(),
            query,
            note,
        };
        self.inner.alerts.write().unwrap().insert(id, alert.clone());
        alert
    }

    pub fn delete_alert(&self, id: &str) -> bool {
        self.inner.alerts.write().unwrap().remove(id).is_some()
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
                let x = self.inner.extras.get(&inst.symbol)?;
                Some(ScreenerRow {
                    // identity / descriptive
                    symbol: inst.symbol.clone(),
                    name: inst.name.clone(),
                    sector: inst.sector.clone(),
                    industry: inst.industry.clone(),
                    exchange: inst.exchange.clone(),
                    country: x.country.clone(),
                    target_price: x.target_price,
                    avg_volume: x.avg_volume,
                    rel_volume: x.rel_volume,
                    float_shares: x.float_shares,
                    recom: x.recom,
                    // market / quote
                    price: q.price,
                    change: q.change,
                    change_pct: q.change_pct,
                    volume: q.volume,
                    // valuation
                    market_cap: f.market_cap,
                    pe: f.pe,
                    forward_pe: x.forward_pe,
                    peg: x.peg,
                    ps: x.ps,
                    pb: x.pb,
                    price_to_fcf: x.price_to_fcf,
                    eps: f.eps,
                    dividend_yield: f.dividend_yield,
                    beta: f.beta,
                    // profitability
                    roa: x.roa,
                    roe: x.roe,
                    roic: x.roic,
                    gross_margin: x.gross_margin,
                    oper_margin: x.oper_margin,
                    profit_margin: x.profit_margin,
                    payout_ratio: x.payout_ratio,
                    // financial health
                    current_ratio: x.current_ratio,
                    quick_ratio: x.quick_ratio,
                    debt_equity: x.debt_equity,
                    lt_debt_equity: x.lt_debt_equity,
                    // ownership
                    insider_own: x.insider_own,
                    inst_own: x.inst_own,
                    short_float: x.short_float,
                    short_ratio: x.short_ratio,
                    // performance
                    perf_week: x.perf_week,
                    perf_month: x.perf_month,
                    perf_quarter: x.perf_quarter,
                    perf_half: x.perf_half,
                    perf_year: x.perf_year,
                    perf_ytd: x.perf_ytd,
                    // technical
                    volatility_w: x.volatility_w,
                    volatility_m: x.volatility_m,
                    rsi14: x.rsi14,
                    atr: x.atr,
                    sma20_rel: x.sma20_rel,
                    sma50_rel: x.sma50_rel,
                    sma200_rel: x.sma200_rel,
                    high_52w_pct: x.high_52w_pct,
                    low_52w_pct: x.low_52w_pct,
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

    // ---- Saved screens -----------------------------------------------------

    /// All saved screens, sorted by name.
    pub fn saved_screens(&self) -> Vec<SavedScreen> {
        let mut v: Vec<SavedScreen> = self
            .inner
            .saved_screens
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    pub fn create_saved_screen(
        &self,
        name: String,
        query: String,
        sort: Option<String>,
        order: Option<String>,
    ) -> SavedScreen {
        let id = format!("ss-{}", self.inner.next_id.fetch_add(1, Ordering::Relaxed));
        let screen = SavedScreen {
            id: id.clone(),
            name,
            query,
            sort,
            order,
        };
        self.inner
            .saved_screens
            .write()
            .unwrap()
            .insert(id, screen.clone());
        screen
    }

    pub fn update_saved_screen(
        &self,
        id: &str,
        name: Option<String>,
        query: Option<String>,
        sort: Option<Option<String>>,
        order: Option<Option<String>>,
    ) -> Option<SavedScreen> {
        let mut guard = self.inner.saved_screens.write().unwrap();
        let screen = guard.get_mut(id)?;
        if let Some(name) = name {
            screen.name = name;
        }
        if let Some(query) = query {
            screen.query = query;
        }
        if let Some(sort) = sort {
            screen.sort = sort;
        }
        if let Some(order) = order {
            screen.order = order;
        }
        Some(screen.clone())
    }

    pub fn delete_saved_screen(&self, id: &str) -> bool {
        self.inner
            .saved_screens
            .write()
            .unwrap()
            .remove(id)
            .is_some()
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

    // ---- News / insider / ratings ------------------------------------------

    /// Synthetic news. With `symbol`, returns that ticker's stream; otherwise a
    /// merged market feed across all instruments. Always newest-first.
    /// Deterministic: identical output on every call for the same arguments.
    pub fn news(&self, symbol: Option<&str>, limit: usize) -> Vec<NewsItem> {
        let base = self.inner.news_base_ts;
        match symbol {
            Some(sym) => {
                let sym = sym.to_ascii_uppercase();
                let Some(inst) = self.inner.instruments.iter().find(|i| i.symbol == sym) else {
                    return Vec::new();
                };
                let Some(quote) = self.inner.quotes.get(&sym) else {
                    return Vec::new();
                };
                news::news_for_symbol(inst, quote, base, limit)
            }
            None => news::market_news(
                &self.inner.instruments,
                |s| self.inner.quotes.get(s).cloned(),
                base,
                limit,
            ),
        }
    }

    /// Synthetic insider trades for a symbol, newest-first. Empty if unknown.
    /// Deterministic across calls.
    pub fn insider_trades(&self, symbol: &str, limit: usize) -> Vec<InsiderTrade> {
        let sym = symbol.to_ascii_uppercase();
        match self.inner.quotes.get(&sym) {
            Some(q) => news::insider_trades(&sym, q, self.inner.news_base_ts, limit),
            None => Vec::new(),
        }
    }

    /// Synthetic analyst ratings for a symbol, newest-first. Empty if unknown.
    /// Deterministic across calls.
    pub fn analyst_ratings(&self, symbol: &str, limit: usize) -> Vec<AnalystRating> {
        let sym = symbol.to_ascii_uppercase();
        match self.inner.quotes.get(&sym) {
            Some(q) => news::analyst_ratings(&sym, q, self.inner.news_base_ts, limit),
            None => Vec::new(),
        }
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
