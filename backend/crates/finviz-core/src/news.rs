//! Deterministic, synthetic news / insider-trade / analyst-rating generators.
//!
//! Like the rest of the seed data, everything here is a pure function of the
//! ticker symbol (and the instrument/quote already in [`crate::AppState`]) plus
//! a per-symbol FNV-seeded LCG. The *set* of items and their ordering are
//! therefore identical on every boot and across repeated calls.
//!
//! Timestamps are anchored to a `base_ts` (the seeded quote timestamp, captured
//! once at boot) and offset *backwards* by deterministic amounts, so the feed
//! always reads "newest first" with stable relative spacing. No wall clock or
//! network input ever influences the generated content.

use finviz_types::{AnalystRating, InsiderTrade, Instrument, NewsItem, Quote};

/// A tiny deterministic FNV-seeded LCG (mirrors the ones in `seed.rs` /
/// `state.rs`), giving each symbol/stream a stable, reproducible draw.
struct Rng {
    state: u64,
}

impl Rng {
    /// Seed from a label (e.g. the ticker symbol) plus a salt so independent
    /// streams (news vs. insider vs. ratings) stay uncorrelated yet deterministic.
    fn new(label: &str, salt: u64) -> Self {
        let mut state = 0xcbf29ce484222325u64; // FNV offset basis
        for b in label.bytes() {
            state = (state ^ b as u64).wrapping_mul(0x100000001b3);
        }
        state ^= salt.wrapping_mul(0x9e3779b97f4a7c15);
        Self { state: state | 1 }
    }

    /// Next value in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 11) as f64) / ((1u64 << 53) as f64)
    }

    /// Next value uniformly in `[lo, hi)`.
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.unit() * (hi - lo)
    }

    /// Pick a deterministic element from a non-empty slice.
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let i = (self.unit() * items.len() as f64) as usize;
        &items[i.min(items.len() - 1)]
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

// ---- small const fragment tables -------------------------------------------

const FIRMS: &[&str] = &[
    "Morgan Stanley",
    "Goldman Sachs",
    "JPMorgan",
    "Bank of America",
    "Wells Fargo",
    "Citigroup",
    "Barclays",
    "UBS",
    "Deutsche Bank",
    "Wedbush",
    "Piper Sandler",
    "Jefferies",
    "Evercore ISI",
    "Raymond James",
];

const RATINGS: &[&str] = &[
    "Buy",
    "Overweight",
    "Outperform",
    "Hold",
    "Neutral",
    "Underweight",
    "Sell",
];

const ACTIONS: &[&str] = &["Upgrade", "Downgrade", "Initiated", "Reiterated"];

const RELATIONS: &[&str] = &[
    "CEO",
    "CFO",
    "COO",
    "Director",
    "President",
    "EVP",
    "10% Owner",
    "General Counsel",
];

const INSIDER_FIRST: &[&str] = &[
    "James", "Mary", "Robert", "Patricia", "Michael", "Linda", "David", "Susan", "Richard",
    "Karen", "Thomas", "Nancy", "Daniel", "Lisa", "Paul",
];

const INSIDER_LAST: &[&str] = &[
    "Carter", "Nguyen", "Patel", "Okafor", "Reyes", "Sullivan", "Bianchi", "Kim", "Larsen",
    "Murphy", "Cohen", "Diaz", "Romano", "Wagner", "Schmidt",
];

const SOURCES: &[&str] = &[
    "Reuters",
    "Bloomberg",
    "MarketWatch",
    "Barron's",
    "CNBC",
    "The Wall Street Journal",
    "Financial Times",
    "Seeking Alpha",
];

/// Headline templates that do not reference a specific firm/quarter.
const GENERIC_TEMPLATES: &[&str] = &[
    "{company} announces $10B share buyback program",
    "{company} expands into new international markets",
    "{company} names new chief operating officer",
    "{company} unveils next-generation product lineup",
    "{company} raises full-year revenue guidance",
    "{company} faces antitrust scrutiny over latest deal",
    "{company} partners with cloud provider on AI initiative",
    "{company} declares quarterly dividend",
];

/// Broad-market headlines (no specific symbol).
const MARKET_TEMPLATES: &[&str] = &[
    "Stocks rally as inflation data cools",
    "Treasury yields edge higher ahead of Fed decision",
    "Tech megacaps lead a broad market advance",
    "Oil prices slip on demand concerns",
    "Dollar strengthens against major peers",
    "Volatility ticks up as earnings season kicks off",
    "Small caps outperform amid rotation into value",
    "Markets mixed as investors weigh rate path",
];

// ---- generators -------------------------------------------------------------

/// Per-symbol news stream, newest-first. `base_ts` is the seeded quote ts.
fn symbol_news(inst: &Instrument, quote: &Quote, base_ts: i64, limit: usize) -> Vec<NewsItem> {
    let mut rng = Rng::new(&inst.symbol, 0xACE1);
    let limit = limit.clamp(1, 100);
    let mut out = Vec::with_capacity(limit);
    // Each item is spaced 4..28h before the previous one, so the stream spans
    // a plausible recent window and stays strictly newest-first.
    let mut ts = base_ts;
    for i in 0..limit {
        ts -= (rng.range(4.0, 28.0) * 3600.0) as i64;
        let (headline, category) = compose_headline(&mut rng, inst, quote);
        let source = rng.pick(SOURCES);
        out.push(NewsItem {
            id: format!("{}-{}", inst.symbol, i),
            ts,
            symbol: Some(inst.symbol.clone()),
            headline,
            source: (*source).to_string(),
            url: format!(
                "https://news.example.com/{}/{i}",
                inst.symbol.to_ascii_lowercase()
            ),
            category: category.to_string(),
        });
    }
    out
}

/// Compose one plausible, varied headline for a symbol and report its category.
fn compose_headline(rng: &mut Rng, inst: &Instrument, quote: &Quote) -> (String, &'static str) {
    // Weighted mix of headline shapes.
    let kind = (rng.unit() * 100.0) as u32;
    let company = inst.name.as_str();
    let sym = inst.symbol.as_str();
    if kind < 28 {
        // Earnings.
        let beat = quote.change_pct >= 0.0;
        let q = 1 + (rng.unit() * 4.0) as u32;
        let verb = if beat { "beats" } else { "misses" };
        (format!("{company} {verb} Q{q} estimates"), "earnings")
    } else if kind < 56 {
        // Analyst.
        let firm = rng.pick(FIRMS);
        let up = rng.unit() < 0.5;
        let verb = if up { "upgrades" } else { "downgrades" };
        let rating = rng.pick(RATINGS);
        (format!("{firm} {verb} {sym} to {rating}"), "analyst")
    } else if kind < 72 {
        // Insider.
        let rel = rng.pick(RELATIONS);
        let act = if rng.unit() < 0.5 { "buys" } else { "sells" };
        (format!("{company} {rel} {act} shares"), "insider")
    } else {
        // Generic corporate.
        let tmpl = rng.pick(GENERIC_TEMPLATES);
        (tmpl.replace("{company}", company), "general")
    }
}

/// Merged market feed across all instruments, newest-first. Interleaves
/// per-symbol items with broad-market headlines.
pub(crate) fn market_news(
    instruments: &[Instrument],
    quote_for: impl Fn(&str) -> Option<Quote>,
    base_ts: i64,
    limit: usize,
) -> Vec<NewsItem> {
    let limit = limit.clamp(1, 200);
    let mut all: Vec<NewsItem> = Vec::new();

    // A few items per instrument so the merged feed is genuinely cross-symbol.
    let per_symbol = 4usize;
    for inst in instruments {
        if let Some(q) = quote_for(&inst.symbol) {
            all.extend(symbol_news(inst, &q, base_ts, per_symbol));
        }
    }

    // Broad-market headlines, seeded independently of any symbol.
    let mut rng = Rng::new("__MARKET__", 0xBEEF);
    let mut ts = base_ts;
    for i in 0..(limit.min(MARKET_TEMPLATES.len() * 2)) {
        ts -= (rng.range(2.0, 10.0) * 3600.0) as i64;
        let headline = rng.pick(MARKET_TEMPLATES);
        let source = rng.pick(SOURCES);
        all.push(NewsItem {
            id: format!("MKT-{i}"),
            ts,
            symbol: None,
            headline: (*headline).to_string(),
            source: (*source).to_string(),
            url: format!("https://news.example.com/markets/{i}"),
            category: "markets".to_string(),
        });
    }

    // Newest-first; ties broken by id for a fully deterministic order.
    all.sort_by(|a, b| b.ts.cmp(&a.ts).then_with(|| a.id.cmp(&b.id)));
    all.truncate(limit);
    all
}

/// Public per-symbol news (sorted newest-first, already from the generator).
pub(crate) fn news_for_symbol(
    inst: &Instrument,
    quote: &Quote,
    base_ts: i64,
    limit: usize,
) -> Vec<NewsItem> {
    symbol_news(inst, quote, base_ts, limit)
}

/// Synthetic insider trades for a symbol, newest-first.
pub(crate) fn insider_trades(
    symbol: &str,
    quote: &Quote,
    base_ts: i64,
    limit: usize,
) -> Vec<InsiderTrade> {
    let mut rng = Rng::new(symbol, 0x1D5E);
    let limit = limit.clamp(1, 50);
    let mut out = Vec::with_capacity(limit);
    let mut ts = base_ts;
    for _ in 0..limit {
        ts -= (rng.range(1.0, 20.0) * 86_400.0) as i64; // 1..20 days apart
        let name = format!("{} {}", rng.pick(INSIDER_FIRST), rng.pick(INSIDER_LAST));
        let relation = (*rng.pick(RELATIONS)).to_string();
        let buy = rng.unit() < 0.5;
        let transaction = if buy { "Buy" } else { "Sell" }.to_string();
        // Trade price within +/-3% of the seeded quote price.
        let price = round2(quote.price * rng.range(0.97, 1.03));
        let shares = (rng.range(500.0, 75_000.0)).round() as i64;
        let value = round2(shares as f64 * price);
        out.push(InsiderTrade {
            symbol: symbol.to_string(),
            insider_name: name,
            relation,
            transaction,
            shares,
            price,
            value,
            ts,
        });
    }
    out
}

/// Synthetic analyst ratings for a symbol, newest-first.
pub(crate) fn analyst_ratings(
    symbol: &str,
    quote: &Quote,
    base_ts: i64,
    limit: usize,
) -> Vec<AnalystRating> {
    let mut rng = Rng::new(symbol, 0x5A71);
    let limit = limit.clamp(1, 50);
    let mut out = Vec::with_capacity(limit);
    let mut ts = base_ts;
    for _ in 0..limit {
        ts -= (rng.range(2.0, 25.0) * 86_400.0) as i64;
        let firm = (*rng.pick(FIRMS)).to_string();
        let action = (*rng.pick(ACTIONS)).to_string();
        let rating = (*rng.pick(RATINGS)).to_string();
        // Price target within roughly -10%..+25% of the seeded price.
        let price_target = Some(round2(quote.price * rng.range(0.90, 1.25)));
        out.push(AnalystRating {
            symbol: symbol.to_string(),
            firm,
            action,
            rating,
            price_target,
            ts,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::AppState;

    fn sorted_desc(ts: &[i64]) -> bool {
        ts.windows(2).all(|w| w[0] >= w[1])
    }

    #[test]
    fn symbol_news_is_deterministic_and_newest_first() {
        let state = AppState::seeded();
        let a = state.news(Some("AAPL"), 25);
        let b = state.news(Some("AAPL"), 25);
        assert_eq!(a, b, "repeated calls must be identical");
        assert_eq!(a.len(), 25);
        let ts: Vec<i64> = a.iter().map(|n| n.ts).collect();
        assert!(sorted_desc(&ts), "news must be newest-first");
        // Every per-symbol item carries that symbol.
        assert!(a.iter().all(|n| n.symbol.as_deref() == Some("AAPL")));
    }

    #[test]
    fn market_news_merges_across_symbols_and_sorts() {
        let state = AppState::seeded();
        let feed = state.news(None, 60);
        let again = state.news(None, 60);
        assert_eq!(feed, again, "market feed must be deterministic");
        assert!(!feed.is_empty());

        let ts: Vec<i64> = feed.iter().map(|n| n.ts).collect();
        assert!(sorted_desc(&ts), "market feed must be newest-first");

        // Merged feed spans more than one ticker plus broad-market items.
        let symbols: std::collections::HashSet<_> =
            feed.iter().filter_map(|n| n.symbol.as_deref()).collect();
        assert!(symbols.len() > 1, "feed should cross multiple symbols");
        assert!(
            feed.iter().any(|n| n.symbol.is_none()),
            "feed should include broad-market headlines"
        );
    }

    #[test]
    fn news_limit_is_clamped() {
        let state = AppState::seeded();
        assert_eq!(state.news(Some("AAPL"), 0).len(), 1, "limit clamps up to 1");
        assert!(
            state.news(None, 100_000).len() <= 200,
            "limit clamps to max"
        );
    }

    #[test]
    fn insider_trades_deterministic_and_value_matches() {
        let state = AppState::seeded();
        let a = state.insider_trades("AAPL", 10);
        let b = state.insider_trades("AAPL", 10);
        assert_eq!(a, b);
        assert_eq!(a.len(), 10);

        let ts: Vec<i64> = a.iter().map(|t| t.ts).collect();
        assert!(sorted_desc(&ts));
        for t in &a {
            assert!(t.transaction == "Buy" || t.transaction == "Sell");
            assert!(t.shares > 0 && t.price > 0.0);
            // value == shares * price within rounding tolerance.
            let expected = t.shares as f64 * t.price;
            assert!(
                (t.value - expected).abs() < 1.0,
                "value should equal shares*price"
            );
        }
    }

    #[test]
    fn analyst_ratings_deterministic_and_plausible() {
        let state = AppState::seeded();
        let a = state.analyst_ratings("AAPL", 8);
        let b = state.analyst_ratings("AAPL", 8);
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);

        let ts: Vec<i64> = a.iter().map(|r| r.ts).collect();
        assert!(sorted_desc(&ts));
        let price = state.quote("AAPL").expect("AAPL seeded").price;
        for r in &a {
            assert!(!r.firm.is_empty() && !r.rating.is_empty());
            let pt = r.price_target.expect("ratings carry a price target");
            // Within the documented -10%..+25% band of the seed price.
            assert!(
                pt >= price * 0.85 && pt <= price * 1.30,
                "price target plausible"
            );
        }
    }

    #[test]
    fn unknown_symbol_yields_empty() {
        let state = AppState::seeded();
        assert!(state.news(Some("NOPE"), 10).is_empty());
        assert!(state.insider_trades("NOPE", 10).is_empty());
        assert!(state.analyst_ratings("NOPE", 10).is_empty());
    }
}
