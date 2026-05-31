//! Pure, deterministic single-symbol backtesting engine.
//!
//! The engine has **no IO and no [`finviz_core`] dependency**: it takes a slice
//! of daily [`Bar`]s in and returns a [`BacktestResult`] out. This keeps it
//! trivially unit-testable and decoupled from the web layer.
//!
//! ## Execution model
//! - The series is walked bar by bar in chronological order.
//! - When **flat**, the entry [`Rule`] is evaluated on every bar. An entry
//!   *fires* on the bar where the rule transitions `false -> true` (an edge, not
//!   a level), so a rule that is already true at series start does not fire.
//! - On the bar after a firing signal we **buy at that next bar's open**. Using
//!   the next open avoids look-ahead bias: the signal is only known once the
//!   triggering bar has closed. If the signal fires on the last bar there is no
//!   next bar and no trade is opened.
//! - While **in a position** the exit conditions are checked on each subsequent
//!   bar; the *earliest* satisfied condition closes the trade at that bar's
//!   close. The conditions are any-of:
//!     - [`Strategy::stop_loss_pct`]: close <= entry_price * (1 - pct/100).
//!     - [`Strategy::time_exit`]: held `bars` bars (entry bar counts as bar 0,
//!       so an exit is forced on entry_index + bars).
//!     - End of series: any open position is closed at the final bar's close.
//!
//! ## Metric formulas (all on a compounding equity curve, start equity = 1.0)
//! For each closed trade with fractional return `r = exit/entry - 1`, equity is
//! multiplied by `(1 + r)`.
//! - `total_return_pct` = `(final_equity - 1) * 100`.
//! - `avg_return_pct`   = mean of per-trade `return_pct`.
//! - `win_rate`         = winning trades (`return_pct > 0`) / total trades.
//! - `max_drawdown_pct` = largest peak-to-trough decline of the equity curve,
//!   `max((peak - equity) / peak)`, reported as a positive percentage.
//! - `sharpe`           = `mean(returns) / stddev(returns)` over per-trade
//!   *fractional* returns (population stddev). Guarded to `0.0` when stddev is
//!   `0` or there are < 2 trades. This is a **raw, non-annualized** ratio.
//! - `calmar`           = `total_return_pct / max_drawdown_pct`, guarded to
//!   `0.0` when max drawdown is `0`. Also raw / non-annualized.
//!
//! All outputs are finite (never `NaN`/`inf`): empty or too-short series yield
//! no trades and all-zero metrics.

use finviz_types::Bar;
use serde::{Deserialize, Serialize};

/// An entry signal rule. Entry fires on the bar where the rule transitions
/// `false -> true`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Rule {
    /// Close is above (or below, if `above = false`) its own `SMA(period)`.
    PriceVsSma { period: usize, above: bool },
    /// `SMA(fast)` is above (golden, `above = true`) or below (death,
    /// `above = false`) `SMA(slow)`. The transition gives the cross.
    SmaCross {
        fast: usize,
        slow: usize,
        #[serde(default = "default_true")]
        above: bool,
    },
    /// `RSI(period)` is below `below` and/or above `above`. When both are set
    /// the rule is true if either bound is satisfied (any-of).
    RsiThreshold {
        period: usize,
        #[serde(default)]
        below: Option<f64>,
        #[serde(default)]
        above: Option<f64>,
    },
}

fn default_true() -> bool {
    true
}

impl Rule {
    /// Validate parameters, returning a human-readable error message on failure.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Rule::PriceVsSma { period, .. } => {
                require_positive("period", *period)?;
            }
            Rule::SmaCross { fast, slow, .. } => {
                require_positive("fast", *fast)?;
                require_positive("slow", *slow)?;
                if fast >= slow {
                    return Err("`fast` period must be strictly less than `slow`".into());
                }
            }
            Rule::RsiThreshold {
                period,
                below,
                above,
            } => {
                require_positive("period", *period)?;
                if below.is_none() && above.is_none() {
                    return Err("RSI threshold needs at least one of `below`/`above`".into());
                }
                for (name, v) in [("below", below), ("above", above)] {
                    if let Some(v) = v {
                        if !v.is_finite() || *v < 0.0 || *v > 100.0 {
                            return Err(format!("RSI `{name}` must be within 0..=100"));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Number of leading bars needed before this rule can produce a value.
    fn warmup(&self) -> usize {
        match self {
            Rule::PriceVsSma { period, .. } => *period,
            Rule::SmaCross { slow, .. } => *slow,
            Rule::RsiThreshold { period, .. } => *period + 1,
        }
    }

    /// Evaluate the rule's boolean state at bar index `i`, or `None` while still
    /// inside the warmup window (state undefined).
    fn state_at(&self, closes: &[f64], i: usize) -> Option<bool> {
        if i + 1 < self.warmup() {
            return None;
        }
        match self {
            Rule::PriceVsSma { period, above } => {
                let sma = sma_at(closes, i, *period)?;
                Some(if *above {
                    closes[i] > sma
                } else {
                    closes[i] < sma
                })
            }
            Rule::SmaCross {
                fast, slow, above, ..
            } => {
                let f = sma_at(closes, i, *fast)?;
                let s = sma_at(closes, i, *slow)?;
                Some(if *above { f > s } else { f < s })
            }
            Rule::RsiThreshold {
                period,
                below,
                above,
            } => {
                let rsi = rsi_at(closes, i, *period)?;
                let lo = below.map(|b| rsi < b).unwrap_or(false);
                let hi = above.map(|a| rsi > a).unwrap_or(false);
                Some(lo || hi)
            }
        }
    }
}

fn require_positive(name: &str, v: usize) -> Result<(), String> {
    if v == 0 {
        Err(format!("`{name}` must be a positive integer"))
    } else {
        Ok(())
    }
}

/// A complete strategy: one entry rule plus optional exit conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    pub entry: Rule,
    /// Hold at most this many bars (entry bar = 0).
    #[serde(default)]
    pub time_exit: Option<usize>,
    /// Exit if close drops this percent below the entry price.
    #[serde(default)]
    pub stop_loss_pct: Option<f64>,
}

impl Strategy {
    /// Validate the whole strategy.
    pub fn validate(&self) -> Result<(), String> {
        self.entry.validate()?;
        if let Some(n) = self.time_exit {
            require_positive("time_exit", n)?;
        }
        if let Some(p) = self.stop_loss_pct {
            if !p.is_finite() || p <= 0.0 || p >= 100.0 {
                return Err("`stop_loss_pct` must be within (0, 100)".into());
            }
        }
        Ok(())
    }
}

/// A single completed round-trip trade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    pub entry_ts: i64,
    pub entry_price: f64,
    pub exit_ts: i64,
    pub exit_price: f64,
    /// Percentage return of this trade, `(exit/entry - 1) * 100`.
    pub return_pct: f64,
    pub bars_held: usize,
}

/// One point on the compounding equity curve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquityPoint {
    pub ts: i64,
    pub equity: f64,
}

/// Aggregate result of a backtest run. All metrics are finite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestResult {
    pub trades: Vec<Trade>,
    pub total_return_pct: f64,
    pub avg_return_pct: f64,
    pub win_rate: f64,
    pub max_drawdown_pct: f64,
    pub sharpe: f64,
    pub calmar: f64,
    pub equity_curve: Vec<EquityPoint>,
}

impl BacktestResult {
    /// An all-zero result with no trades (used for empty/short series).
    fn empty() -> Self {
        BacktestResult {
            trades: Vec::new(),
            total_return_pct: 0.0,
            avg_return_pct: 0.0,
            win_rate: 0.0,
            max_drawdown_pct: 0.0,
            sharpe: 0.0,
            calmar: 0.0,
            equity_curve: vec![EquityPoint { ts: 0, equity: 1.0 }],
        }
    }
}

/// Run the backtest. See the module docs for the execution model and metric
/// formulas. Pure and deterministic.
pub fn run_backtest(bars: &[Bar], strategy: &Strategy) -> BacktestResult {
    if bars.len() < 2 {
        let mut empty = BacktestResult::empty();
        if let Some(first) = bars.first() {
            empty.equity_curve[0].ts = first.ts;
        }
        return empty;
    }

    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let trades = simulate(bars, &closes, strategy);
    let equity_curve = build_equity_curve(bars, &trades);
    finalize(trades, equity_curve)
}

/// Core simulation loop. Returns the list of completed trades.
fn simulate(bars: &[Bar], closes: &[f64], strategy: &Strategy) -> Vec<Trade> {
    let mut trades = Vec::new();
    let rule = &strategy.entry;
    let n = bars.len();

    let mut i = 0usize;
    let mut prev_state: Option<bool> = None;

    while i < n {
        // Look for an entry edge (false -> true) using the *closing* state of
        // bar `i`; enter on the next bar's open.
        let state = rule.state_at(closes, i);
        let fired = matches!((prev_state, state), (Some(false), Some(true)));
        prev_state = state;

        if fired && i + 1 < n {
            let entry_idx = i + 1;
            let entry_price = bars[entry_idx].open;
            let (exit_idx, exit_price) = resolve_exit(bars, strategy, entry_idx, entry_price);
            let return_pct = (exit_price / entry_price - 1.0) * 100.0;
            trades.push(Trade {
                entry_ts: bars[entry_idx].ts,
                entry_price,
                exit_ts: bars[exit_idx].ts,
                exit_price,
                return_pct,
                bars_held: exit_idx - entry_idx,
            });
            // Resume scanning from the bar after the exit; reset edge detection.
            i = exit_idx + 1;
            prev_state = rule.state_at(closes, exit_idx);
            continue;
        }
        i += 1;
    }
    trades
}

/// Find the exit bar for a position opened at `entry_idx` / `entry_price`.
/// Returns `(exit_index, exit_price)`.
fn resolve_exit(
    bars: &[Bar],
    strategy: &Strategy,
    entry_idx: usize,
    entry_price: f64,
) -> (usize, f64) {
    let n = bars.len();
    let stop_level = strategy
        .stop_loss_pct
        .map(|pct| entry_price * (1.0 - pct / 100.0));

    let mut j = entry_idx + 1;
    while j < n {
        let close = bars[j].close;
        if let Some(level) = stop_level {
            if close <= level {
                return (j, close);
            }
        }
        if let Some(hold) = strategy.time_exit {
            if j - entry_idx >= hold {
                return (j, close);
            }
        }
        j += 1;
    }
    // Ran off the end: close at the final bar.
    let last = n - 1;
    (last, bars[last].close)
}

/// Build the compounding equity curve. Equity starts at 1.0 on the first bar
/// and steps at each trade's exit timestamp.
fn build_equity_curve(bars: &[Bar], trades: &[Trade]) -> Vec<EquityPoint> {
    let mut curve = Vec::with_capacity(trades.len() + 1);
    let start_ts = bars.first().map(|b| b.ts).unwrap_or(0);
    curve.push(EquityPoint {
        ts: start_ts,
        equity: 1.0,
    });
    let mut equity = 1.0;
    for t in trades {
        equity *= 1.0 + t.return_pct / 100.0;
        curve.push(EquityPoint {
            ts: t.exit_ts,
            equity,
        });
    }
    curve
}

/// Compute aggregate metrics from trades + equity curve.
fn finalize(trades: Vec<Trade>, equity_curve: Vec<EquityPoint>) -> BacktestResult {
    if trades.is_empty() {
        let mut r = BacktestResult::empty();
        r.equity_curve = equity_curve;
        return r;
    }

    let count = trades.len() as f64;
    let final_equity = equity_curve.last().map(|p| p.equity).unwrap_or(1.0);
    let total_return_pct = (final_equity - 1.0) * 100.0;

    let avg_return_pct = trades.iter().map(|t| t.return_pct).sum::<f64>() / count;
    let wins = trades.iter().filter(|t| t.return_pct > 0.0).count();
    let win_rate = wins as f64 / count;

    let max_drawdown_pct = max_drawdown(&equity_curve);

    // Sharpe on fractional per-trade returns (population stddev), raw.
    let fracs: Vec<f64> = trades.iter().map(|t| t.return_pct / 100.0).collect();
    let mean = fracs.iter().sum::<f64>() / count;
    let sharpe = if fracs.len() < 2 {
        0.0
    } else {
        let var = fracs.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / count;
        let sd = var.sqrt();
        if sd == 0.0 {
            0.0
        } else {
            mean / sd
        }
    };

    let calmar = if max_drawdown_pct == 0.0 {
        0.0
    } else {
        total_return_pct / max_drawdown_pct
    };

    BacktestResult {
        trades,
        total_return_pct,
        avg_return_pct,
        win_rate,
        max_drawdown_pct,
        sharpe,
        calmar,
        equity_curve,
    }
}

/// Largest peak-to-trough decline of the equity curve, as a positive percent.
fn max_drawdown(curve: &[EquityPoint]) -> f64 {
    let mut peak = f64::MIN;
    let mut max_dd = 0.0;
    for p in curve {
        if p.equity > peak {
            peak = p.equity;
        }
        if peak > 0.0 {
            let dd = (peak - p.equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd * 100.0
}

// ---- shared indicator math (same style as the API's `indicators.rs`) ----

/// `SMA(period)` ending at index `i`, or `None` if there aren't enough bars.
fn sma_at(closes: &[f64], i: usize, period: usize) -> Option<f64> {
    if period == 0 || i + 1 < period {
        return None;
    }
    let window = &closes[i + 1 - period..=i];
    Some(window.iter().sum::<f64>() / period as f64)
}

/// Wilder's `RSI(period)` value at index `i`, or `None` if there aren't enough
/// bars. A zero average loss yields an RSI of 100.
fn rsi_at(closes: &[f64], i: usize, period: usize) -> Option<f64> {
    if period == 0 || i < period {
        return None;
    }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for k in 1..=period {
        let diff = closes[k] - closes[k - 1];
        if diff >= 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;
    for k in (period + 1)..=i {
        let diff = closes[k] - closes[k - 1];
        let (gain, loss) = if diff >= 0.0 {
            (diff, 0.0)
        } else {
            (0.0, -diff)
        };
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
    }
    Some(if avg_loss == 0.0 {
        100.0
    } else {
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(ts: i64, close: f64) -> Bar {
        // open == prior close in our crafted series isn't required; tests that
        // depend on entry price set `open` explicitly via `bar_oc`.
        Bar {
            ts,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1,
        }
    }

    fn bar_oc(ts: i64, open: f64, close: f64) -> Bar {
        Bar {
            ts,
            open,
            high: open.max(close),
            low: open.min(close),
            close,
            volume: 1,
        }
    }

    fn series(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| bar(i as i64, c))
            .collect()
    }

    #[test]
    fn empty_and_short_series_yield_no_trades_and_zero_metrics() {
        for n in 0..2 {
            let bars: Vec<Bar> = (0..n).map(|i| bar(i, 100.0)).collect();
            let strat = Strategy {
                entry: Rule::SmaCross {
                    fast: 2,
                    slow: 3,
                    above: true,
                },
                time_exit: None,
                stop_loss_pct: None,
            };
            let r = run_backtest(&bars, &strat);
            assert!(r.trades.is_empty());
            assert_eq!(r.total_return_pct, 0.0);
            assert_eq!(r.win_rate, 0.0);
            assert_eq!(r.max_drawdown_pct, 0.0);
            assert_eq!(r.sharpe, 0.0);
            assert_eq!(r.calmar, 0.0);
            assert!(r.total_return_pct.is_finite());
            assert!(r.sharpe.is_finite());
        }
    }

    #[test]
    fn sma_cross_on_rising_series_yields_winning_trade() {
        // Dip then steady rise: fast SMA crosses above slow SMA, then price
        // keeps rising so the trade is a winner.
        let closes = [
            10.0, 9.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ];
        let bars = series(&closes);
        let strat = Strategy {
            entry: Rule::SmaCross {
                fast: 2,
                slow: 3,
                above: true,
            },
            time_exit: None,
            stop_loss_pct: None,
        };
        let r = run_backtest(&bars, &strat);
        assert!(!r.trades.is_empty(), "expected at least one trade");
        let winners = r.trades.iter().filter(|t| t.return_pct > 0.0).count();
        assert!(winners >= 1, "expected >=1 winning trade, got {winners}");
        assert!(r.total_return_pct > 0.0);
        assert!(r.win_rate > 0.0);
    }

    #[test]
    fn stop_loss_triggers_on_crafted_drop() {
        // Enter via price-vs-sma, then a sharp drop must trigger the 5% stop.
        // closes: warmup rising, cross, then crash.
        let closes = [
            10.0, 10.0, 10.0, 11.0, 12.0, // entry edge around here (above sma)
            12.0, 6.0, // -50% crash -> stop hit
            6.0,
        ];
        // Make the post-signal open a known value so entry price is determinate.
        let mut bars = series(&closes);
        // entry happens at bar after the cross; ensure its open == its close set above.
        for b in &mut bars {
            b.open = b.close;
        }
        let strat = Strategy {
            entry: Rule::PriceVsSma {
                period: 3,
                above: true,
            },
            time_exit: None,
            stop_loss_pct: Some(5.0),
        };
        let r = run_backtest(&bars, &strat);
        assert!(!r.trades.is_empty());
        let t = &r.trades[0];
        // Exit price should be the crash close (6.0), far below entry.
        assert!(t.return_pct < -5.0, "stop should produce a big loss: {t:?}");
        assert!(r.max_drawdown_pct > 0.0);
    }

    #[test]
    fn time_exit_closes_after_n_bars() {
        // Start below SMA (rule false), then cross above (false->true edge),
        // then keep rising; hold exactly 2 bars.
        let closes = [5.0, 4.0, 3.0, 4.0, 6.0, 7.0, 8.0, 9.0];
        let bars = series(&closes);
        let strat = Strategy {
            entry: Rule::PriceVsSma {
                period: 2,
                above: true,
            },
            time_exit: Some(2),
            stop_loss_pct: None,
        };
        let r = run_backtest(&bars, &strat);
        assert!(!r.trades.is_empty());
        assert_eq!(
            r.trades[0].bars_held, 2,
            "time exit must hold exactly 2 bars"
        );
    }

    #[test]
    fn win_rate_and_drawdown_on_hand_built_case() {
        // Two trades: +10% then -20%, compounding.
        //   equity: 1.0 -> 1.10 -> 0.88
        //   total_return = -12%
        //   win_rate = 1/2 = 0.5
        //   max_drawdown: peak 1.10, trough 0.88 -> (1.10-0.88)/1.10 = 20%
        let trades = vec![
            Trade {
                entry_ts: 0,
                entry_price: 100.0,
                exit_ts: 1,
                exit_price: 110.0,
                return_pct: 10.0,
                bars_held: 1,
            },
            Trade {
                entry_ts: 2,
                entry_price: 100.0,
                exit_ts: 3,
                exit_price: 80.0,
                return_pct: -20.0,
                bars_held: 1,
            },
        ];
        let curve = build_equity_curve(&series(&[1.0, 1.0, 1.0, 1.0]), &trades);
        let r = finalize(trades, curve);
        assert!(
            (r.total_return_pct - (-12.0)).abs() < 1e-9,
            "{}",
            r.total_return_pct
        );
        assert!((r.win_rate - 0.5).abs() < 1e-9);
        assert!(
            (r.max_drawdown_pct - 20.0).abs() < 1e-9,
            "{}",
            r.max_drawdown_pct
        );
        assert!((r.avg_return_pct - (-5.0)).abs() < 1e-9);
        // calmar = total_return / max_dd = -12/20 = -0.6
        assert!((r.calmar - (-0.6)).abs() < 1e-9, "{}", r.calmar);
        // sharpe finite and negative (mean of fractional returns is negative).
        assert!(r.sharpe.is_finite());
        assert!(r.sharpe < 0.0);
    }

    #[test]
    fn entry_buys_at_next_bar_open() {
        // Build a series where the cross fires on bar i, and bar i+1 has a
        // distinctive open we can assert against.
        let mut bars = series(&[10.0, 9.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0]);
        // Give the next-open a sentinel: set open of every bar to close+0.5.
        for b in &mut bars {
            b.open = b.close + 0.5;
        }
        let strat = Strategy {
            entry: Rule::SmaCross {
                fast: 2,
                slow: 3,
                above: true,
            },
            time_exit: Some(1),
            stop_loss_pct: None,
        };
        let r = run_backtest(&bars, &strat);
        assert!(!r.trades.is_empty());
        let t = &r.trades[0];
        // entry price equals the open (close+0.5) of its bar.
        let entry_bar = bars.iter().find(|b| b.ts == t.entry_ts).unwrap();
        assert!((t.entry_price - entry_bar.open).abs() < 1e-9);
    }

    #[test]
    fn rsi_threshold_below_fires_entry() {
        // Rise first (RSI high, rule false), then a steady decline pushes RSI
        // below 30 (false->true edge), then recovery for the exit.
        let mut closes = vec![100.0];
        for _ in 0..7 {
            let last = *closes.last().unwrap();
            closes.push(last * 1.03); // rising -> RSI high
        }
        for _ in 0..8 {
            let last = *closes.last().unwrap();
            closes.push(last * 0.95); // steady decline -> low RSI
        }
        for _ in 0..6 {
            let last = *closes.last().unwrap();
            closes.push(last * 1.05); // recovery
        }
        let bars: Vec<Bar> = closes
            .iter()
            .enumerate()
            .map(|(i, &c)| bar_oc(i as i64, c, c))
            .collect();
        let strat = Strategy {
            entry: Rule::RsiThreshold {
                period: 5,
                below: Some(30.0),
                above: None,
            },
            time_exit: Some(4),
            stop_loss_pct: None,
        };
        let r = run_backtest(&bars, &strat);
        assert!(!r.trades.is_empty(), "RSI<30 should produce an entry");
    }

    #[test]
    fn validation_rejects_bad_params() {
        assert!(Rule::PriceVsSma {
            period: 0,
            above: true
        }
        .validate()
        .is_err());
        assert!(Rule::SmaCross {
            fast: 5,
            slow: 5,
            above: true
        }
        .validate()
        .is_err());
        assert!(Rule::RsiThreshold {
            period: 14,
            below: None,
            above: None
        }
        .validate()
        .is_err());
        assert!(Rule::RsiThreshold {
            period: 14,
            below: Some(150.0),
            above: None
        }
        .validate()
        .is_err());
        let bad_stop = Strategy {
            entry: Rule::PriceVsSma {
                period: 3,
                above: true,
            },
            time_exit: None,
            stop_loss_pct: Some(0.0),
        };
        assert!(bad_stop.validate().is_err());
        let good = Strategy {
            entry: Rule::SmaCross {
                fast: 5,
                slow: 20,
                above: true,
            },
            time_exit: Some(10),
            stop_loss_pct: Some(8.0),
        };
        assert!(good.validate().is_ok());
    }
}
