//! Deterministic, synthetic market boards: futures, forex and crypto.
//!
//! Like the rest of the seed data, every row is a pure function of its symbol
//! plus a per-symbol FNV-seeded LCG that mirrors the ones in `seed.rs`,
//! `news.rs` and `derivatives.rs`. The output is therefore identical on every
//! boot and across repeated calls — no wall clock or network input ever
//! influences the generated content.
//!
//! ## Prices are illustrative, NOT real market data
//!
//! Each board is built from a small const table of realistic instruments with
//! plausible base prices; the per-session `change` and the trailing `perf_week`
//! / `perf_month` figures are then jittered deterministically from the symbol.
//! `change_pct` is derived from `change` and `price` so its sign always matches
//! `change`. These numbers are for UI demonstration only.

use finviz_types::MarketAsset;

/// A tiny deterministic FNV-seeded LCG (mirrors `seed.rs` / `derivatives.rs`).
struct Rng {
    state: u64,
}

impl Rng {
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
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// One row of a board's base table: `(symbol, name, group, base_price, decimals)`.
///
/// `decimals` controls price/change rounding so FX rates keep their extra
/// precision (e.g. 4 dp) while index/crypto prices round to 2 dp.
struct Spec {
    symbol: &'static str,
    name: &'static str,
    group: &'static str,
    base_price: f64,
    decimals: i32,
}

/// Round to `decimals` decimal places.
fn round_to(x: f64, decimals: i32) -> f64 {
    let f = 10f64.powi(decimals);
    (x * f).round() / f
}

/// Turn one base spec into a fully-populated [`MarketAsset`], deterministically.
///
/// The day's move is drawn as a percent in roughly `[-2.5%, +2.5%]`; `change`
/// is `price` minus the implied prior close, and `change_pct` is recomputed
/// from `change / prev_close` so its sign matches `change` exactly. Weekly and
/// monthly performance are anchored to the day's direction (so a green day
/// trends with a green week) plus independent seeded noise.
fn build(spec: &Spec) -> MarketAsset {
    let mut rng = Rng::new(spec.symbol, 0x804D);

    let price = spec.base_price;
    // Day move as a fraction of price, +/- ~2.5%.
    let move_pct = rng.range(-2.5, 2.5);
    let prev_close = price / (1.0 + move_pct / 100.0);
    let change = round_to(price - prev_close, spec.decimals);
    // Recompute pct from the rounded change so the sign is always consistent.
    let change_pct = if prev_close != 0.0 {
        round2(change / prev_close * 100.0)
    } else {
        0.0
    };

    // Trailing performance: anchored to today's direction, then seeded noise.
    let perf_week = round2(change_pct * rng.range(1.0, 3.0) + rng.range(-3.0, 3.0));
    let perf_month = round2(perf_week + rng.range(-7.0, 9.0));

    MarketAsset {
        symbol: spec.symbol.to_string(),
        name: spec.name.to_string(),
        group: spec.group.to_string(),
        price: round_to(price, spec.decimals),
        change,
        change_pct,
        perf_week,
        perf_month,
    }
}

/// Build a full board from its const spec table, preserving table order.
fn board(specs: &[Spec]) -> Vec<MarketAsset> {
    specs.iter().map(build).collect()
}

// ---- Futures ---------------------------------------------------------------

const FUTURES: &[Spec] = &[
    // Equity index futures.
    Spec {
        symbol: "ES",
        name: "E-mini S&P 500",
        group: "Indices",
        base_price: 5_280.00,
        decimals: 2,
    },
    Spec {
        symbol: "NQ",
        name: "E-mini Nasdaq 100",
        group: "Indices",
        base_price: 18_650.00,
        decimals: 2,
    },
    Spec {
        symbol: "YM",
        name: "E-mini Dow Jones",
        group: "Indices",
        base_price: 39_120.00,
        decimals: 2,
    },
    Spec {
        symbol: "RTY",
        name: "E-mini Russell 2000",
        group: "Indices",
        base_price: 2_070.00,
        decimals: 2,
    },
    // Energy.
    Spec {
        symbol: "CL",
        name: "Crude Oil WTI",
        group: "Energy",
        base_price: 78.40,
        decimals: 2,
    },
    Spec {
        symbol: "NG",
        name: "Natural Gas",
        group: "Energy",
        base_price: 2.65,
        decimals: 3,
    },
    Spec {
        symbol: "RB",
        name: "RBOB Gasoline",
        group: "Energy",
        base_price: 2.48,
        decimals: 4,
    },
    // Metals.
    Spec {
        symbol: "GC",
        name: "Gold",
        group: "Metals",
        base_price: 2_345.00,
        decimals: 2,
    },
    Spec {
        symbol: "SI",
        name: "Silver",
        group: "Metals",
        base_price: 29.80,
        decimals: 3,
    },
    Spec {
        symbol: "HG",
        name: "Copper",
        group: "Metals",
        base_price: 4.52,
        decimals: 4,
    },
    // Agriculture.
    Spec {
        symbol: "ZC",
        name: "Corn",
        group: "Agriculture",
        base_price: 442.50,
        decimals: 2,
    },
    Spec {
        symbol: "ZW",
        name: "Wheat",
        group: "Agriculture",
        base_price: 596.25,
        decimals: 2,
    },
    Spec {
        symbol: "ZS",
        name: "Soybeans",
        group: "Agriculture",
        base_price: 1_188.00,
        decimals: 2,
    },
];

/// All synthetic futures rows, in table order. Deterministic; prices illustrative.
pub(crate) fn futures() -> Vec<MarketAsset> {
    board(FUTURES)
}

// ---- Forex -----------------------------------------------------------------

const FOREX: &[Spec] = &[
    // Majors (USD pairs).
    Spec {
        symbol: "EURUSD",
        name: "Euro / US Dollar",
        group: "Major",
        base_price: 1.0850,
        decimals: 4,
    },
    Spec {
        symbol: "USDJPY",
        name: "US Dollar / Japanese Yen",
        group: "Major",
        base_price: 157.30,
        decimals: 3,
    },
    Spec {
        symbol: "GBPUSD",
        name: "British Pound / US Dollar",
        group: "Major",
        base_price: 1.2720,
        decimals: 4,
    },
    Spec {
        symbol: "USDCHF",
        name: "US Dollar / Swiss Franc",
        group: "Major",
        base_price: 0.8980,
        decimals: 4,
    },
    Spec {
        symbol: "AUDUSD",
        name: "Australian Dollar / US Dollar",
        group: "Major",
        base_price: 0.6640,
        decimals: 4,
    },
    Spec {
        symbol: "USDCAD",
        name: "US Dollar / Canadian Dollar",
        group: "Major",
        base_price: 1.3680,
        decimals: 4,
    },
    Spec {
        symbol: "NZDUSD",
        name: "New Zealand Dollar / US Dollar",
        group: "Major",
        base_price: 0.6120,
        decimals: 4,
    },
    // Crosses (non-USD).
    Spec {
        symbol: "EURGBP",
        name: "Euro / British Pound",
        group: "Minor",
        base_price: 0.8530,
        decimals: 4,
    },
    Spec {
        symbol: "EURJPY",
        name: "Euro / Japanese Yen",
        group: "Minor",
        base_price: 170.65,
        decimals: 3,
    },
];

/// All synthetic forex rows, in table order. Deterministic; rates illustrative.
pub(crate) fn forex() -> Vec<MarketAsset> {
    board(FOREX)
}

// ---- Crypto ----------------------------------------------------------------

const CRYPTO: &[Spec] = &[
    Spec {
        symbol: "BTCUSD",
        name: "Bitcoin",
        group: "Crypto",
        base_price: 67_500.00,
        decimals: 2,
    },
    Spec {
        symbol: "ETHUSD",
        name: "Ethereum",
        group: "Crypto",
        base_price: 3_520.00,
        decimals: 2,
    },
    Spec {
        symbol: "SOLUSD",
        name: "Solana",
        group: "Crypto",
        base_price: 168.40,
        decimals: 2,
    },
    Spec {
        symbol: "XRPUSD",
        name: "XRP",
        group: "Crypto",
        base_price: 0.5230,
        decimals: 4,
    },
    Spec {
        symbol: "ADAUSD",
        name: "Cardano",
        group: "Crypto",
        base_price: 0.4480,
        decimals: 4,
    },
    Spec {
        symbol: "DOGEUSD",
        name: "Dogecoin",
        group: "Crypto",
        base_price: 0.1530,
        decimals: 5,
    },
    Spec {
        symbol: "BNBUSD",
        name: "BNB",
        group: "Crypto",
        base_price: 592.00,
        decimals: 2,
    },
    Spec {
        symbol: "AVAXUSD",
        name: "Avalanche",
        group: "Crypto",
        base_price: 36.20,
        decimals: 3,
    },
    Spec {
        symbol: "LINKUSD",
        name: "Chainlink",
        group: "Crypto",
        base_price: 17.85,
        decimals: 3,
    },
];

/// All synthetic crypto rows, in table order. Deterministic; prices illustrative.
pub(crate) fn crypto() -> Vec<MarketAsset> {
    board(CRYPTO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Shared invariants for any board.
    fn assert_board_well_formed(rows: &[MarketAsset]) {
        assert!(!rows.is_empty(), "board must be non-empty");

        let mut symbols = HashSet::new();
        for r in rows {
            assert!(
                symbols.insert(r.symbol.clone()),
                "duplicate symbol {}",
                r.symbol
            );
            assert!(!r.group.is_empty(), "group populated for {}", r.symbol);
            assert!(!r.name.is_empty(), "name populated for {}", r.symbol);
            assert!(r.price > 0.0, "positive price for {}", r.symbol);
            // change_pct sign must agree with change sign.
            assert_eq!(
                r.change.partial_cmp(&0.0),
                r.change_pct.partial_cmp(&0.0),
                "change/change_pct sign mismatch for {}",
                r.symbol
            );
        }
    }

    #[test]
    fn futures_board_is_deterministic_and_well_formed() {
        let a = futures();
        let b = futures();
        assert_eq!(a, b, "repeated calls must be identical");
        assert_board_well_formed(&a);
        // All four expected futures groups are represented.
        for g in ["Indices", "Energy", "Metals", "Agriculture"] {
            assert!(a.iter().any(|r| r.group == g), "missing futures group {g}");
        }
    }

    #[test]
    fn forex_board_is_deterministic_and_well_formed() {
        let a = forex();
        let b = forex();
        assert_eq!(a, b, "repeated calls must be identical");
        assert_board_well_formed(&a);
        assert!(a.iter().any(|r| r.group == "Major"));
        assert!(a.iter().any(|r| r.group == "Minor"));
    }

    #[test]
    fn crypto_board_is_deterministic_and_well_formed() {
        let a = crypto();
        let b = crypto();
        assert_eq!(a, b, "repeated calls must be identical");
        assert_board_well_formed(&a);
        assert!(a.iter().all(|r| r.group == "Crypto"));
    }
}
