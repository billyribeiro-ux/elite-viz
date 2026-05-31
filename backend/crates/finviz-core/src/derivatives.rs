//! Deterministic, synthetic options chains and ETF profiles.
//!
//! Like the rest of the seed data, everything here is a pure function of the
//! ticker symbol (and the instrument / quote / fundamentals already in
//! [`crate::AppState`]) plus a per-symbol FNV-seeded LCG that mirrors the ones
//! in `seed.rs` / `news.rs`. The output is therefore identical on every boot
//! and across repeated calls — no wall clock or network input ever influences
//! the generated content.
//!
//! ## Pricing is illustrative, NOT a real model
//!
//! The option premiums, implied vols and deltas below are a deliberately crude
//! "intrinsic value + linear time value" sketch chosen to produce *plausible,
//! internally-consistent* numbers (bid ≤ last ≤ ask, ATM delta ≈ ±0.5, deeper
//! ITM → |delta| → 1, deeper OTM → |delta| → 0). They are **not** Black-Scholes
//! and must not be used for anything beyond UI demonstration.

use finviz_types::{
    EtfHolding, EtfProfile, Fundamentals, Instrument, OptionChain, OptionContract, Quote,
};

/// A tiny deterministic FNV-seeded LCG (mirrors `news.rs` / `state.rs`).
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

// ---- Options ---------------------------------------------------------------

/// Clamp helpers shared with the route layer's defaults.
pub const EXPIRIES_MAX: usize = 8;
pub const STRIKES_MAX: usize = 20;

/// A simple civil-date type so we can roll expiries forward deterministically
/// without pulling in a date library. Only the (small) forward range we need is
/// supported.
#[derive(Clone, Copy)]
struct Ymd {
    y: i64,
    m: i64,
}

impl Ymd {
    /// Advance by `n` whole months.
    fn add_months(self, n: i64) -> Ymd {
        let total = (self.y * 12 + (self.m - 1)) + n;
        Ymd {
            y: total.div_euclid(12),
            m: total.rem_euclid(12) + 1,
        }
    }
}

/// Anchor for forward expiry dates. Fixed (not wall-clock) so the chain is
/// fully deterministic across boots, matching the seed-data philosophy.
const EXPIRY_ANCHOR: Ymd = Ymd { y: 2026, m: 1 };

/// The "third Friday" of a month is the canonical US monthly-options expiry.
/// Computed via Zeller-style day-of-week of the 1st, then the offset to the
/// first Friday, plus 14 days.
fn third_friday(ymd: Ymd) -> (i64, i64, i64) {
    // Day of week of the 1st (0=Sat .. per Zeller). We map to ISO-ish Friday.
    let (mut y, mut m) = (ymd.y, ymd.m);
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;
    // Zeller's congruence for the 1st of the month: h: 0=Sat,1=Sun,...,6=Fri.
    let q = 1i64;
    let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j).rem_euclid(7);
    // Friday is h == 6. Days to the first Friday from the 1st:
    let first_friday = ((6 - h).rem_euclid(7)) + 1;
    let day = first_friday + 14; // third Friday
    (ymd.y, ymd.m, day)
}

fn expiry_string(i: usize) -> String {
    let ymd = EXPIRY_ANCHOR.add_months(i as i64);
    let (y, m, d) = third_friday(ymd);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Pick a "nice" strike increment from the underlying price (a coarse version
/// of how real chains widen their strike grid with price).
fn strike_increment(price: f64) -> f64 {
    if price < 25.0 {
        0.5
    } else if price < 100.0 {
        1.0
    } else if price < 250.0 {
        2.5
    } else if price < 500.0 {
        5.0
    } else {
        10.0
    }
}

/// OCC-style contract symbol, e.g. `AAPL260116C00210000`.
fn contract_id(symbol: &str, expiry: &str, kind_char: char, strike: f64) -> String {
    // expiry is YYYY-MM-DD -> YYMMDD
    let digits: String = expiry.chars().filter(|c| c.is_ascii_digit()).collect();
    let yymmdd = digits.get(2..).unwrap_or("000000");
    let strike_milli = (strike * 1000.0).round() as i64;
    format!("{symbol}{yymmdd}{kind_char}{strike_milli:08}")
}

/// Build a synthetic option chain for `symbol`.
///
/// For each of `expiries` monthly expirations and each of `strikes_per_side`
/// strikes above and below the underlying, emit one call and one put. The
/// contract count is therefore `expiries * strikes_per_side * 2`.
///
/// Illustrative pricing (NOT Black-Scholes):
/// * `t` = years to expiry ≈ `(expiry_index + 1) / 12`.
/// * `iv` = a per-symbol base vol (25%..65%) plus a small volatility "smile"
///   that rises with moneyness distance.
/// * time value ≈ `underlying * iv * sqrt(t) * exp(-2 * moneyness^2)` (a
///   bell-shaped peak at-the-money, decaying as the strike moves away).
/// * `last` = `intrinsic + time_value`; `bid`/`ask` straddle `last` by a small
///   per-contract spread; `last` is re-clamped into `[bid, ask]`.
/// * `delta`: ATM ≈ ±0.5, tending to ±1 deep ITM and 0 deep OTM, via a smooth
///   logistic in moneyness.
pub(crate) fn option_chain(
    inst: &Instrument,
    quote: &Quote,
    expiries: usize,
    strikes_per_side: usize,
) -> OptionChain {
    let symbol = inst.symbol.as_str();
    let expiries = expiries.clamp(1, EXPIRIES_MAX);
    let strikes_per_side = strikes_per_side.clamp(1, STRIKES_MAX);

    let price = quote.price.max(0.01);
    let inc = strike_increment(price);
    // Center the ladder on the nearest increment to the underlying.
    let center = (price / inc).round() * inc;

    let mut rng = Rng::new(symbol, 0x0971);
    // Per-symbol base implied vol, stable across the whole chain.
    let base_iv = rng.range(0.25, 0.65);

    let mut expiry_list: Vec<String> = Vec::with_capacity(expiries);
    let mut contracts: Vec<OptionContract> = Vec::with_capacity(expiries * strikes_per_side * 2);

    for ei in 0..expiries {
        let expiry = expiry_string(ei);
        expiry_list.push(expiry.clone());
        let t = (ei as f64 + 1.0) / 12.0; // years to expiry
        let sqrt_t = t.sqrt();

        // Strikes from -strikes_per_side .. +strikes_per_side (excluding the
        // center duplicate handling: we include the center as offset 0 once).
        let lo = -(strikes_per_side as i64);
        let hi = strikes_per_side as i64;
        for k in lo..=hi {
            if k == 0 {
                continue; // keep exactly strikes_per_side on each side
            }
            // We want strikes_per_side strikes per side, so map k=1..=N above
            // and k=-1..=-N below; the `continue` above drops the center.
            let strike = round2(center + k as f64 * inc);
            if strike <= 0.0 {
                continue;
            }

            // Moneyness: log of strike/price (0 at the money).
            let moneyness = (strike / price).ln();
            // Volatility smile: vol rises as we move away from the money.
            let iv = (base_iv + 0.20 * moneyness.abs()).clamp(0.05, 2.0);
            // Bell-shaped time value, peaking ATM.
            let time_value = price * iv * sqrt_t * (-2.0 * moneyness * moneyness).exp();

            for (kind, kind_char) in [("call", 'C'), ("put", 'P')] {
                let intrinsic = if kind == "call" {
                    (price - strike).max(0.0)
                } else {
                    (strike - price).max(0.0)
                };
                let mut last = round2(intrinsic + time_value);

                // Per-contract deterministic spread and quote sizes.
                let mut cr = Rng::new(&format!("{symbol}|{expiry}|{kind}|{strike}"), 0x0D7A);
                let half_spread = round2((0.5 + last * 0.02) * (0.5 + cr.unit()));
                let bid = round2((last - half_spread).max(0.0));
                let ask = round2(last + half_spread);
                // Keep last within [bid, ask].
                if last < bid {
                    last = bid;
                }
                if last > ask {
                    last = ask;
                }

                // Volume/OI fall off away from the money (more ATM activity).
                let activity = (-1.5 * moneyness * moneyness).exp();
                let volume = (cr.range(0.0, 8000.0) * activity).round() as i64;
                let open_interest = (cr.range(500.0, 60_000.0) * activity).round() as i64;

                // Delta: smooth logistic in moneyness; ATM ≈ ±0.5.
                let call_delta = 1.0 / (1.0 + (-moneyness / (iv * sqrt_t)).exp());
                let delta = if kind == "call" {
                    call_delta
                } else {
                    call_delta - 1.0
                };

                contracts.push(OptionContract {
                    contract: contract_id(symbol, &expiry, kind_char, strike),
                    kind: kind.to_string(),
                    strike,
                    expiry: expiry.clone(),
                    bid,
                    ask,
                    last: round2(last),
                    volume: volume.max(0),
                    open_interest: open_interest.max(0),
                    implied_vol: round2(iv),
                    delta: (delta * 10000.0).round() / 10000.0,
                });
            }
        }
    }

    OptionChain {
        symbol: symbol.to_string(),
        underlying_price: price,
        expiries: expiry_list,
        contracts,
    }
}

// ---- ETFs ------------------------------------------------------------------

/// Designated synthetic ETFs: `(symbol, name, category, expense_ratio %, aum)`
/// plus a sector filter used to pick holdings. A `None` sector means "broad
/// market" (top names by market cap regardless of sector).
struct EtfSpec {
    symbol: &'static str,
    name: &'static str,
    category: &'static str,
    expense_ratio: f64,
    aum: f64,
    sector: Option<&'static str>,
    holdings: usize,
}

const ETFS: &[EtfSpec] = &[
    EtfSpec {
        symbol: "SPY",
        name: "SPDR S&P 500 ETF Trust",
        category: "Large Blend",
        expense_ratio: 0.09,
        aum: 5.20e11,
        sector: None,
        holdings: 15,
    },
    EtfSpec {
        symbol: "QQQ",
        name: "Invesco QQQ Trust",
        category: "Large Growth",
        expense_ratio: 0.20,
        aum: 2.70e11,
        sector: None,
        holdings: 12,
    },
    EtfSpec {
        symbol: "DIA",
        name: "SPDR Dow Jones Industrial Average ETF Trust",
        category: "Large Value",
        expense_ratio: 0.16,
        aum: 3.40e10,
        sector: None,
        holdings: 10,
    },
    EtfSpec {
        symbol: "IWM",
        name: "iShares Russell 2000 ETF",
        category: "Small Blend",
        expense_ratio: 0.19,
        aum: 6.50e10,
        sector: None,
        holdings: 12,
    },
    EtfSpec {
        symbol: "XLK",
        name: "Technology Select Sector SPDR Fund",
        category: "Technology",
        expense_ratio: 0.09,
        aum: 7.10e10,
        sector: Some("Technology"),
        holdings: 8,
    },
];

/// `true` if `symbol` (case-insensitive) is one of the designated ETFs.
pub(crate) fn is_etf(symbol: &str) -> bool {
    let sym = symbol.to_ascii_uppercase();
    ETFS.iter().any(|e| e.symbol == sym)
}

/// All designated ETF symbols (ascending), for discovery.
pub(crate) fn etf_symbols() -> Vec<String> {
    let mut v: Vec<String> = ETFS.iter().map(|e| e.symbol.to_string()).collect();
    v.sort();
    v
}

/// Build the profile for one designated ETF, weighting holdings by market cap
/// over the (optionally sector-filtered) seed universe and normalizing the
/// weights to sum to ≈ 100%. Returns `None` for non-ETF symbols.
pub(crate) fn etf_profile(
    symbol: &str,
    instruments: &[Instrument],
    cap_for: impl Fn(&str) -> Option<f64>,
) -> Option<EtfProfile> {
    let sym = symbol.to_ascii_uppercase();
    let spec = ETFS.iter().find(|e| e.symbol == sym)?;

    // Candidate constituents: (instrument, market_cap), optionally by sector.
    let mut candidates: Vec<(&Instrument, f64)> = instruments
        .iter()
        .filter(|i| spec.sector.is_none_or(|s| i.sector == s))
        .filter_map(|i| cap_for(&i.symbol).map(|c| (i, c.max(0.0))))
        .collect();

    // Heaviest first; ties broken by symbol for determinism.
    candidates.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.symbol.cmp(&b.0.symbol))
    });
    candidates.truncate(spec.holdings.min(candidates.len()));

    let total: f64 = candidates.iter().map(|(_, c)| c).sum();
    let mut holdings: Vec<EtfHolding> = if total > 0.0 {
        candidates
            .iter()
            .map(|(inst, cap)| EtfHolding {
                symbol: inst.symbol.clone(),
                name: inst.name.clone(),
                weight: round2(cap / total * 100.0),
            })
            .collect()
    } else {
        // Degenerate fallback: equal weight.
        let n = candidates.len().max(1) as f64;
        candidates
            .iter()
            .map(|(inst, _)| EtfHolding {
                symbol: inst.symbol.clone(),
                name: inst.name.clone(),
                weight: round2(100.0 / n),
            })
            .collect()
    };

    // Absorb any rounding residual into the largest holding so the reported
    // weights sum to exactly 100.00.
    let sum: f64 = holdings.iter().map(|h| h.weight).sum();
    if let Some(first) = holdings.first_mut() {
        first.weight = round2(first.weight + (100.0 - sum));
    }

    Some(EtfProfile {
        symbol: spec.symbol.to_string(),
        name: spec.name.to_string(),
        holdings,
        expense_ratio: spec.expense_ratio,
        aum: spec.aum,
        category: spec.category.to_string(),
    })
}

/// All designated ETF profiles, ascending by symbol.
pub(crate) fn etf_profiles(
    instruments: &[Instrument],
    cap_for: impl Fn(&str) -> Option<f64> + Copy,
) -> Vec<EtfProfile> {
    let mut out: Vec<EtfProfile> = etf_symbols()
        .iter()
        .filter_map(|s| etf_profile(s, instruments, cap_for))
        .collect();
    out.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    out
}

/// Convenience: market cap lookup from a fundamentals option.
pub(crate) fn cap_of(f: Option<&Fundamentals>) -> Option<f64> {
    f.map(|f| f.market_cap)
}

#[cfg(test)]
mod tests {
    use crate::AppState;

    #[test]
    fn option_chain_is_deterministic_and_well_formed() {
        let state = AppState::seeded();
        let a = state.option_chain("AAPL", 4, 8).expect("AAPL seeded");
        let b = state.option_chain("AAPL", 4, 8).expect("AAPL seeded");
        assert_eq!(a, b, "repeated calls must be identical");

        // `strikes_per_side` strikes ABOVE and BELOW the money (2 sides), each
        // with a call and a put: expiries * strikes_per_side * 2 sides * 2 kinds.
        assert_eq!(a.expiries.len(), 4);
        assert_eq!(a.contracts.len(), 4 * 8 * 2 * 2);
        // Per expiry: 2 * strikes_per_side strikes, one call + one put each.
        let calls = a.contracts.iter().filter(|c| c.kind == "call").count();
        let puts = a.contracts.iter().filter(|c| c.kind == "put").count();
        assert_eq!(calls, puts, "one call per put");

        for c in &a.contracts {
            assert!(c.kind == "call" || c.kind == "put");
            assert!(c.bid <= c.ask, "bid <= ask for {}", c.contract);
            assert!(
                c.last >= c.bid - 1e-9 && c.last <= c.ask + 1e-9,
                "last within [bid, ask] for {}",
                c.contract
            );
            assert!(c.strike > 0.0);
            assert!(c.implied_vol > 0.0);
            if c.kind == "call" {
                assert!((0.0..=1.0).contains(&c.delta), "call delta in 0..1");
            } else {
                assert!((-1.0..=0.0).contains(&c.delta), "put delta in -1..0");
            }
        }
    }

    #[test]
    fn atm_call_delta_is_near_half() {
        let state = AppState::seeded();
        let chain = state.option_chain("AAPL", 1, 10).expect("AAPL seeded");
        let price = chain.underlying_price;
        // The near-the-money call: smallest |strike - price| among calls.
        let atm = chain
            .contracts
            .iter()
            .filter(|c| c.kind == "call")
            .min_by(|a, b| {
                (a.strike - price)
                    .abs()
                    .partial_cmp(&(b.strike - price).abs())
                    .unwrap()
            })
            .expect("at least one call");
        assert!(
            (atm.delta - 0.5).abs() < 0.15,
            "ATM call delta ≈ 0.5, got {}",
            atm.delta
        );
    }

    #[test]
    fn unknown_symbol_has_no_chain() {
        let state = AppState::seeded();
        assert!(state.option_chain("ZZZZ", 4, 8).is_none());
    }

    #[test]
    fn etf_profile_weights_sum_to_100() {
        let state = AppState::seeded();
        let spy = state.etf_profile("SPY").expect("SPY is an ETF");
        assert!(!spy.holdings.is_empty());
        let sum: f64 = spy.holdings.iter().map(|h| h.weight).sum();
        assert!((sum - 100.0).abs() < 0.5, "weights sum ≈ 100, got {sum}");
        // Case-insensitive and deterministic.
        assert_eq!(spy, state.etf_profile("spy").expect("case-insensitive"));
        assert_eq!(spy, state.etf_profile("SPY").unwrap());
    }

    #[test]
    fn non_etf_has_no_profile() {
        let state = AppState::seeded();
        // AAPL is a seeded single stock, not a designated ETF.
        assert!(state.etf_profile("AAPL").is_none());
        assert!(!state.is_etf("AAPL"));
        assert!(state.etf_profile("ZZZZ").is_none());
    }

    #[test]
    fn etfs_listing_is_non_empty_and_sorted() {
        let state = AppState::seeded();
        let etfs = state.etfs();
        assert!(!etfs.is_empty());
        let symbols: Vec<&str> = etfs.iter().map(|e| e.symbol.as_str()).collect();
        let mut sorted = symbols.clone();
        sorted.sort_unstable();
        assert_eq!(symbols, sorted, "ETF list must be sorted by symbol");
        // Each listed ETF round-trips through etf_profile.
        for e in &etfs {
            assert_eq!(Some(e.clone()), state.etf_profile(&e.symbol));
        }
    }
}
