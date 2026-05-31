//! A small, clearly-synthetic seed dataset so the platform runs with no
//! database and no upstream market-data credentials. Values are illustrative,
//! not real market data.

use finviz_types::{Fundamentals, Instrument, Quote};

/// One seed record: reference data + a quote + fundamentals + the extended
/// FINVIZ-style metric surface the screener filters on.
pub struct SeedRow {
    pub instrument: Instrument,
    pub quote: Quote,
    pub fundamentals: Fundamentals,
    pub extras: ScreenerExtras,
}

/// The extended (non-quote, non-core-fundamental) screener metrics. Every value
/// is synthesized deterministically from the base seed row, so it is stable
/// across runs (no wall-clock or network input). See [`derive_extras`].
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenerExtras {
    // descriptive
    pub country: String,
    pub target_price: Option<f64>,
    pub avg_volume: f64,
    pub rel_volume: f64,
    pub float_shares: f64,
    pub recom: Option<f64>,
    // valuation
    pub forward_pe: Option<f64>,
    pub peg: Option<f64>,
    pub ps: Option<f64>,
    pub pb: Option<f64>,
    pub price_to_fcf: Option<f64>,
    // profitability
    pub roa: Option<f64>,
    pub roe: Option<f64>,
    pub roic: Option<f64>,
    pub gross_margin: Option<f64>,
    pub oper_margin: Option<f64>,
    pub profit_margin: Option<f64>,
    pub payout_ratio: Option<f64>,
    // financial health
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub debt_equity: Option<f64>,
    pub lt_debt_equity: Option<f64>,
    // ownership
    pub insider_own: Option<f64>,
    pub inst_own: Option<f64>,
    pub short_float: Option<f64>,
    pub short_ratio: Option<f64>,
    // performance
    pub perf_week: f64,
    pub perf_month: f64,
    pub perf_quarter: f64,
    pub perf_half: f64,
    pub perf_year: f64,
    pub perf_ytd: f64,
    // technical
    pub volatility_w: f64,
    pub volatility_m: f64,
    pub rsi14: f64,
    pub atr: f64,
    pub sma20_rel: f64,
    pub sma50_rel: f64,
    pub sma200_rel: f64,
    pub high_52w_pct: f64,
    pub low_52w_pct: f64,
}

/// A tiny deterministic FNV-seeded LCG (mirrors the one in `state.rs`), used to
/// give each symbol a stable, reproducible spread of synthetic metric values.
struct SeedRng {
    state: u64,
}

impl SeedRng {
    /// Seed from the ticker symbol plus a salt so independent metric families
    /// draw from uncorrelated streams while staying deterministic.
    fn new(symbol: &str, salt: u64) -> Self {
        let mut state = 0xcbf29ce484222325u64; // FNV offset basis
        for b in symbol.bytes() {
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

/// Derive the full extended metric set for one symbol from its base seed.
///
/// All values are plausible and internally consistent: growth/tech names skew
/// to richer multiples and stronger trailing performance, value/defensive names
/// to cheaper multiples and steadier technicals. Margins, RSI, ownership, etc.
/// are clamped to realistic ranges. Everything is a pure function of the inputs
/// plus a per-symbol seeded RNG, so it is identical on every boot.
#[allow(clippy::too_many_arguments)]
fn derive_extras(
    symbol: &str,
    sector: &str,
    price: f64,
    change_pct: f64,
    volume: i64,
    cap: f64,
    pe: f64,
    eps: f64,
    div_yield: f64,
) -> ScreenerExtras {
    let mut rng = SeedRng::new(symbol, 0x01);
    let growthy = sector == "Technology" || sector == "Communication Services";
    let defensive = sector == "Consumer Defensive" || sector == "Healthcare";
    let profitable = pe > 0.0 && eps > 0.0;

    // --- descriptive ---
    // The seed is a US large-cap universe.
    let country = "USA".to_string();
    // Analyst price target: a few percent above current, more upside for growth.
    let upside = if growthy {
        rng.range(0.02, 0.22)
    } else {
        rng.range(-0.05, 0.12)
    };
    let target_price = profitable.then(|| round2(price * (1.0 + upside)));
    // Avg volume drifts a bit from today's volume; rel_volume is the ratio.
    let avg_volume = (volume as f64 * rng.range(0.8, 1.25)).round();
    let rel_volume = if avg_volume > 0.0 {
        round2(volume as f64 / avg_volume)
    } else {
        1.0
    };
    let shares = if price > 0.0 { cap / price } else { 0.0 };
    // Float: 80-99% of shares outstanding (insiders/locked-up shares excluded).
    let float_shares = (shares * rng.range(0.80, 0.99)).round();
    // Recommendation 1..5; growth names trend toward "buy" (lower).
    let recom = profitable.then(|| {
        if growthy {
            round2(rng.range(1.4, 2.6))
        } else {
            round2(rng.range(1.8, 3.4))
        }
    });

    // --- valuation ---
    // Forward P/E a touch below trailing for growers, above for shrinkers.
    let forward_pe = (pe > 0.0).then(|| {
        let factor = if growthy {
            rng.range(0.80, 0.98)
        } else {
            rng.range(0.92, 1.08)
        };
        round2(pe * factor)
    });
    // Earnings growth implied by PEG; growthy names grow faster.
    let growth = if growthy {
        rng.range(15.0, 40.0)
    } else {
        rng.range(3.0, 14.0)
    };
    let peg = (pe > 0.0 && growth > 0.0).then(|| round2(pe / growth));
    // Price/Sales, Price/Book, Price/FCF: richer for growth, cheaper for value.
    let ps = Some(round2(if growthy {
        rng.range(4.0, 14.0)
    } else {
        rng.range(0.6, 4.5)
    }));
    let pb = Some(round2(if growthy {
        rng.range(4.0, 22.0)
    } else {
        rng.range(0.8, 6.0)
    }));
    let price_to_fcf = profitable.then(|| {
        round2(if growthy {
            rng.range(20.0, 60.0)
        } else {
            rng.range(8.0, 30.0)
        })
    });

    // --- profitability (percent) ---
    let profit_margin = profitable.then(|| {
        round2(if growthy {
            rng.range(12.0, 38.0)
        } else {
            rng.range(3.0, 20.0)
        })
    });
    let oper_margin = profit_margin.map(|pm| round2(pm * rng.range(1.05, 1.4)));
    let gross_margin = Some(round2(if growthy {
        rng.range(45.0, 80.0)
    } else {
        rng.range(20.0, 55.0)
    }));
    let roe = profitable.then(|| {
        round2(if growthy {
            rng.range(18.0, 55.0)
        } else {
            rng.range(6.0, 28.0)
        })
    });
    let roa = roe.map(|e| round2(e * rng.range(0.25, 0.6)));
    let roic = roe.map(|e| round2(e * rng.range(0.6, 0.95)));
    // Payout ratio tracks dividend policy.
    let payout_ratio = (div_yield > 0.0).then(|| round2(rng.range(15.0, 70.0)));

    // --- financial health ---
    let current_ratio = Some(round2(rng.range(0.8, 3.5)));
    let quick_ratio = current_ratio.map(|c| round2((c * rng.range(0.55, 0.9)).max(0.2)));
    let debt_equity = Some(round2(if defensive {
        rng.range(0.1, 1.2)
    } else {
        rng.range(0.0, 2.2)
    }));
    let lt_debt_equity = debt_equity.map(|d| round2(d * rng.range(0.6, 0.95)));

    // --- ownership (percent) ---
    let insider_own = Some(round2(rng.range(0.05, 8.0)));
    let inst_own = Some(round2(rng.range(45.0, 92.0)));
    let short_float = Some(round2(rng.range(0.4, 18.0)));
    let short_ratio = short_float.map(|_| round2(rng.range(0.5, 7.0)));

    // --- performance (percent) ---
    // Anchor longer windows to today's move so direction is coherent, then add
    // per-window seeded noise. Growth names get a wider, more positive spread.
    let bias = if growthy { 6.0 } else { 0.0 };
    let perf_week = round2(change_pct * rng.range(1.0, 3.0) + rng.range(-4.0, 4.0));
    let perf_month = round2(perf_week + rng.range(-8.0, 12.0) + bias * 0.3);
    let perf_quarter = round2(perf_month + rng.range(-12.0, 20.0) + bias * 0.5);
    let perf_half = round2(perf_quarter + rng.range(-15.0, 28.0) + bias);
    let perf_year = round2((perf_half + rng.range(-20.0, 45.0) + bias * 2.0).clamp(-60.0, 120.0));
    let perf_ytd = round2(perf_half * rng.range(0.5, 1.1));

    // --- technical ---
    let volatility_w = round2(rng.range(1.0, 6.0) + if growthy { 1.5 } else { 0.0 });
    let volatility_m = round2(volatility_w * rng.range(1.0, 1.6));
    let rsi14 = round2(rng.range(20.0, 80.0));
    // ATR scales with price and weekly volatility.
    let atr = round2((price * volatility_w / 100.0).max(0.01));
    // SMA distances cohere with trailing performance (uptrend => price above).
    let sma20_rel = round2((perf_month * 0.25 + rng.range(-3.0, 3.0)).clamp(-25.0, 25.0));
    let sma50_rel = round2((perf_quarter * 0.25 + rng.range(-5.0, 5.0)).clamp(-40.0, 40.0));
    let sma200_rel = round2((perf_year * 0.4 + rng.range(-8.0, 8.0)).clamp(-60.0, 90.0));
    // 52-week high is at-or-above price (pct <= 0); low at-or-below (pct >= 0).
    let high_52w_pct = round2(-rng.range(0.0, 35.0));
    let low_52w_pct = round2(rng.range(5.0, 110.0));

    ScreenerExtras {
        country,
        target_price,
        avg_volume,
        rel_volume,
        float_shares,
        recom,
        forward_pe,
        peg,
        ps,
        pb,
        price_to_fcf,
        roa,
        roe,
        roic,
        gross_margin,
        oper_margin,
        profit_margin,
        payout_ratio,
        current_ratio,
        quick_ratio,
        debt_equity,
        lt_debt_equity,
        insider_own,
        inst_own,
        short_float,
        short_ratio,
        perf_week,
        perf_month,
        perf_quarter,
        perf_half,
        perf_year,
        perf_ytd,
        volatility_w,
        volatility_m,
        rsi14,
        atr,
        sma20_rel,
        sma50_rel,
        sma200_rel,
        high_52w_pct,
        low_52w_pct,
    }
}

/// Build the seed dataset. `now` is the quote timestamp (epoch seconds).
#[allow(clippy::type_complexity)] // a flat tuple table is the clearest form for seed data
pub fn dataset(now: i64) -> Vec<SeedRow> {
    // (symbol, name, sector, industry, exchange, price, prev_close, volume,
    //  market_cap, pe, eps, div_yield, beta)
    const RAW: &[(
        &str,
        &str,
        &str,
        &str,
        &str,
        f64,
        f64,
        i64,
        f64,
        f64,
        f64,
        f64,
        f64,
    )] = &[
        (
            "AAPL",
            "Apple Inc.",
            "Technology",
            "Consumer Electronics",
            "NASDAQ",
            212.45,
            209.80,
            54_300_000,
            3.30e12,
            33.1,
            6.42,
            0.45,
            1.20,
        ),
        (
            "MSFT",
            "Microsoft Corp.",
            "Technology",
            "Software—Infrastructure",
            "NASDAQ",
            441.20,
            438.10,
            18_900_000,
            3.28e12,
            37.4,
            11.80,
            0.70,
            0.92,
        ),
        (
            "NVDA",
            "NVIDIA Corp.",
            "Technology",
            "Semiconductors",
            "NASDAQ",
            124.30,
            119.55,
            240_100_000,
            3.05e12,
            65.2,
            1.91,
            0.03,
            1.75,
        ),
        (
            "GOOGL",
            "Alphabet Inc.",
            "Communication Services",
            "Internet Content",
            "NASDAQ",
            178.60,
            176.90,
            22_400_000,
            2.18e12,
            27.8,
            6.42,
            0.00,
            1.05,
        ),
        (
            "AMZN",
            "Amazon.com Inc.",
            "Consumer Cyclical",
            "Internet Retail",
            "NASDAQ",
            186.10,
            183.40,
            41_200_000,
            1.94e12,
            51.0,
            3.65,
            0.00,
            1.15,
        ),
        (
            "META",
            "Meta Platforms Inc.",
            "Communication Services",
            "Internet Content",
            "NASDAQ",
            498.30,
            505.20,
            14_700_000,
            1.26e12,
            28.9,
            17.24,
            0.40,
            1.22,
        ),
        (
            "TSLA",
            "Tesla Inc.",
            "Consumer Cyclical",
            "Auto Manufacturers",
            "NASDAQ",
            182.50,
            177.30,
            98_600_000,
            5.81e11,
            44.7,
            4.08,
            0.00,
            2.30,
        ),
        (
            "BRK.B",
            "Berkshire Hathaway",
            "Financial Services",
            "Insurance—Diversified",
            "NYSE",
            412.80,
            410.10,
            3_100_000,
            8.90e11,
            9.8,
            42.10,
            0.00,
            0.86,
        ),
        (
            "JPM",
            "JPMorgan Chase & Co.",
            "Financial Services",
            "Banks—Diversified",
            "NYSE",
            198.40,
            196.70,
            9_800_000,
            5.70e11,
            11.6,
            17.10,
            2.30,
            1.10,
        ),
        (
            "V",
            "Visa Inc.",
            "Financial Services",
            "Credit Services",
            "NYSE",
            274.10,
            272.50,
            6_200_000,
            5.55e11,
            30.2,
            9.08,
            0.75,
            0.95,
        ),
        (
            "JNJ",
            "Johnson & Johnson",
            "Healthcare",
            "Drug Manufacturers",
            "NYSE",
            146.20,
            147.10,
            7_400_000,
            3.52e11,
            22.4,
            6.53,
            3.30,
            0.55,
        ),
        (
            "WMT",
            "Walmart Inc.",
            "Consumer Defensive",
            "Discount Stores",
            "NYSE",
            67.30,
            66.85,
            15_300_000,
            5.42e11,
            28.7,
            2.34,
            1.25,
            0.50,
        ),
        (
            "PG",
            "Procter & Gamble",
            "Consumer Defensive",
            "Household Products",
            "NYSE",
            165.40,
            164.20,
            5_900_000,
            3.90e11,
            26.1,
            6.34,
            2.40,
            0.42,
        ),
        (
            "XOM",
            "Exxon Mobil Corp.",
            "Energy",
            "Oil & Gas Integrated",
            "NYSE",
            113.70,
            115.20,
            16_100_000,
            4.51e11,
            13.5,
            8.42,
            3.20,
            0.90,
        ),
        (
            "CVX",
            "Chevron Corp.",
            "Energy",
            "Oil & Gas Integrated",
            "NYSE",
            156.30,
            158.10,
            8_700_000,
            2.88e11,
            14.2,
            11.01,
            4.10,
            1.02,
        ),
        (
            "UNH",
            "UnitedHealth Group",
            "Healthcare",
            "Healthcare Plans",
            "NYSE",
            492.10,
            488.40,
            3_300_000,
            4.55e11,
            19.8,
            24.85,
            1.50,
            0.65,
        ),
        (
            "HD",
            "Home Depot Inc.",
            "Consumer Cyclical",
            "Home Improvement",
            "NYSE",
            342.60,
            339.90,
            3_600_000,
            3.40e11,
            22.9,
            14.96,
            2.45,
            1.04,
        ),
        (
            "BAC",
            "Bank of America",
            "Financial Services",
            "Banks—Diversified",
            "NYSE",
            39.80,
            39.20,
            38_400_000,
            3.10e11,
            12.3,
            3.24,
            2.65,
            1.30,
        ),
        (
            "KO",
            "Coca-Cola Co.",
            "Consumer Defensive",
            "Beverages—Non-Alcoholic",
            "NYSE",
            62.10,
            61.80,
            13_900_000,
            2.68e11,
            24.6,
            2.52,
            3.05,
            0.58,
        ),
        (
            "PEP",
            "PepsiCo Inc.",
            "Consumer Defensive",
            "Beverages—Non-Alcoholic",
            "NASDAQ",
            168.40,
            169.10,
            5_100_000,
            2.31e11,
            23.1,
            7.29,
            3.10,
            0.52,
        ),
        (
            "INTC",
            "Intel Corp.",
            "Technology",
            "Semiconductors",
            "NASDAQ",
            30.20,
            31.05,
            44_700_000,
            1.28e11,
            0.0,
            -0.45,
            1.65,
            1.08,
        ),
        (
            "AMD",
            "Advanced Micro Devices",
            "Technology",
            "Semiconductors",
            "NASDAQ",
            162.80,
            158.40,
            51_200_000,
            2.63e11,
            0.0,
            0.62,
            0.00,
            1.70,
        ),
        (
            "NFLX",
            "Netflix Inc.",
            "Communication Services",
            "Entertainment",
            "NASDAQ",
            648.20,
            640.50,
            4_900_000,
            2.79e11,
            44.2,
            14.66,
            0.00,
            1.28,
        ),
        (
            "DIS",
            "Walt Disney Co.",
            "Communication Services",
            "Entertainment",
            "NYSE",
            101.30,
            102.80,
            11_200_000,
            1.84e11,
            36.4,
            2.78,
            0.85,
            1.40,
        ),
        (
            "BA",
            "Boeing Co.",
            "Industrials",
            "Aerospace & Defense",
            "NYSE",
            178.50,
            181.20,
            7_800_000,
            1.09e11,
            0.0,
            -3.67,
            0.00,
            1.55,
        ),
        (
            "PFE",
            "Pfizer Inc.",
            "Healthcare",
            "Drug Manufacturers",
            "NYSE",
            28.40,
            28.10,
            33_600_000,
            1.61e11,
            17.9,
            1.59,
            5.90,
            0.60,
        ),
    ];

    RAW.iter()
        .map(
            |&(sym, name, sector, industry, exch, price, prev, vol, cap, pe, eps, dy, beta)| {
                let change = price - prev;
                let change_pct = if prev != 0.0 {
                    change / prev * 100.0
                } else {
                    0.0
                };
                SeedRow {
                    instrument: Instrument {
                        symbol: sym.into(),
                        name: name.into(),
                        sector: sector.into(),
                        industry: industry.into(),
                        exchange: exch.into(),
                    },
                    quote: Quote {
                        symbol: sym.into(),
                        price,
                        change,
                        change_pct,
                        volume: vol,
                        prev_close: prev,
                        day_high: price.max(prev) * 1.012,
                        day_low: price.min(prev) * 0.988,
                        ts: now,
                    },
                    fundamentals: Fundamentals {
                        symbol: sym.into(),
                        market_cap: cap,
                        pe: (pe > 0.0).then_some(pe),
                        eps: Some(eps),
                        dividend_yield: (dy > 0.0).then_some(dy),
                        beta: Some(beta),
                        shares_outstanding: if price > 0.0 { cap / price } else { 0.0 },
                    },
                    extras: derive_extras(sym, sector, price, change_pct, vol, cap, pe, eps, dy),
                }
            },
        )
        .collect()
}
