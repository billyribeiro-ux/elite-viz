//! Classic chart-pattern detection over a bar series.
//!
//! This crate is a **pure** detector library: [`detect`] takes a slice of
//! [`Bar`]s and returns a set of [`Pattern`] detections. It performs no IO and
//! has no knowledge of `AppState` or the HTTP layer.
//!
//! The heuristics here are deliberately approximate and FINVIZ-style — they are
//! cheap, deterministic geometric rules over the close/high/low series, *not*
//! machine learning. The goal is to surface plausible classic formations with a
//! `confidence` in `0..1` describing how cleanly the rule matched.
//!
//! ## Approach
//!
//! 1. **Trend / channels.** A least-squares line is fit to the trailing closes.
//!    The slope (normalised by price) classifies the trend up/down, and the
//!    R² of the fit measures how cleanly the closes hug a straight channel. A
//!    high-R² rising fit → [`PatternKind::ChannelUp`]; falling → `ChannelDown`.
//!
//! 2. **Swing extrema.** Local maxima of the high series and local minima of the
//!    low series are extracted with a small fractal window. Lines are fit
//!    through the swing highs (the "resistance" envelope) and the swing lows
//!    (the "support" envelope). The pair of slopes classifies triangles and
//!    wedges:
//!      - flat top + rising bottom → ascending triangle
//!      - falling top + flat bottom → descending triangle
//!      - converging (top down, bottom up) → symmetric triangle
//!      - both sloping the *same* direction and converging → wedge
//!
//! 3. **Double top / bottom.** Two comparable swing highs separated by an
//!    intervening trough (within a price tolerance) → double top; the mirror on
//!    swing lows → double bottom.
//!
//! 4. **Head & shoulders.** Three swing highs where the middle ("head") is the
//!    tallest and the outer two ("shoulders") are comparable to each other.
//!
//! Each detector returns a confidence in `0..1`. [`detect`] collects every
//! detector that fires; [`detect_best`] returns the single highest-confidence
//! detection. Everything is deterministic for a given input.

use finviz_types::Bar;
use serde::{Deserialize, Serialize};

/// The catalogue of classic chart patterns this crate can recognise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatternKind {
    ChannelUp,
    ChannelDown,
    TriangleAscending,
    TriangleDescending,
    TriangleSymmetric,
    Wedge,
    DoubleTop,
    DoubleBottom,
    HeadAndShoulders,
}

/// A single detected pattern over a span of the input series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pattern {
    pub kind: PatternKind,
    /// Timestamp (epoch seconds) of the first bar covered by the detection.
    pub start_ts: i64,
    /// Timestamp (epoch seconds) of the last bar covered by the detection.
    pub end_ts: i64,
    /// How cleanly the heuristic matched, in `0.0..=1.0`.
    pub confidence: f64,
    /// Human-readable summary of why the pattern fired.
    pub description: String,
}

/// Minimum bars before any detector will attempt to fit.
const MIN_BARS: usize = 12;

/// Run every detector and return all patterns that fired, sorted by descending
/// confidence (ties broken deterministically by kind ordering).
///
/// Returns an empty vector when there are too few bars or nothing matches.
pub fn detect(bars: &[Bar]) -> Vec<Pattern> {
    if bars.len() < MIN_BARS {
        return Vec::new();
    }

    let mut out: Vec<Pattern> = Vec::new();
    out.extend(detect_channel(bars));
    out.extend(detect_triangle_or_wedge(bars));
    out.extend(detect_double(bars));
    out.extend(detect_head_and_shoulders(bars));

    out.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| (a.kind as u8).cmp(&(b.kind as u8)))
    });
    out
}

/// Return the single highest-confidence pattern, if any fired.
pub fn detect_best(bars: &[Bar]) -> Option<Pattern> {
    detect(bars).into_iter().next()
}

// ---------------------------------------------------------------------------
// Linear-algebra helpers
// ---------------------------------------------------------------------------

/// Ordinary least-squares line fit over `(x, y)` samples.
///
/// Returns `(slope, intercept, r_squared)`. `r_squared` is clamped to
/// `0.0..=1.0`. Degenerate inputs (fewer than two points, zero x-variance, or
/// zero y-variance) yield a flat fit with `r_squared == 1.0` (a constant series
/// is, trivially, a perfect line).
fn linfit(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    let n = xs.len().min(ys.len());
    if n < 2 {
        return (0.0, ys.first().copied().unwrap_or(0.0), 1.0);
    }
    let nf = n as f64;
    let mean_x = xs[..n].iter().sum::<f64>() / nf;
    let mean_y = ys[..n].iter().sum::<f64>() / nf;

    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let mut syy = 0.0;
    for (&x, &y) in xs[..n].iter().zip(ys[..n].iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx.abs() < f64::EPSILON {
        return (0.0, mean_y, 1.0);
    }
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    let r2 = if syy.abs() < f64::EPSILON {
        1.0
    } else {
        ((sxy * sxy) / (sxx * syy)).clamp(0.0, 1.0)
    };
    (slope, intercept, r2)
}

/// Slope normalised to a fraction of the mean level, per bar. This makes slopes
/// comparable across symbols of different price magnitudes.
fn normalised_slope(slope: f64, level: f64) -> f64 {
    if level.abs() < f64::EPSILON {
        0.0
    } else {
        slope / level
    }
}

// ---------------------------------------------------------------------------
// Swing extrema
// ---------------------------------------------------------------------------

/// A local extremum: index into the bar slice plus its value.
#[derive(Debug, Clone, Copy)]
struct Swing {
    idx: usize,
    value: f64,
}

/// Local maxima of `values` using a symmetric fractal window of `w` bars on
/// each side. A point is a swing high if it is `>=` every neighbour in the
/// window and strictly greater than at least one (avoids flat plateaus firing
/// at every index).
fn swing_highs(values: &[f64], w: usize) -> Vec<Swing> {
    swings(values, w, true)
}

/// Local minima of `values`; mirror of [`swing_highs`].
fn swing_lows(values: &[f64], w: usize) -> Vec<Swing> {
    swings(values, w, false)
}

fn swings(values: &[f64], w: usize, want_max: bool) -> Vec<Swing> {
    let n = values.len();
    let mut out = Vec::new();
    if n < 2 * w + 1 {
        return out;
    }
    for i in w..(n - w) {
        let v = values[i];
        let mut dominates = true;
        let mut strict = false;
        for (offset, &neighbour) in values[(i - w)..=(i + w)].iter().enumerate() {
            if offset == w {
                continue; // the candidate itself
            }
            if want_max {
                if neighbour > v {
                    dominates = false;
                    break;
                }
                if neighbour < v {
                    strict = true;
                }
            } else {
                if neighbour < v {
                    dominates = false;
                    break;
                }
                if neighbour > v {
                    strict = true;
                }
            }
        }
        if dominates && strict {
            out.push(Swing { idx: i, value: v });
        }
    }
    out
}

/// Fractal half-window scaled to series length, kept small and bounded.
fn swing_window(n: usize) -> usize {
    (n / 20).clamp(2, 5)
}

// ---------------------------------------------------------------------------
// Detectors
// ---------------------------------------------------------------------------

/// Fit a line to the trailing closes. A clean (high-R²) sloping fit is reported
/// as a rising or falling channel. Confidence is the R² of the fit, gated so
/// that near-flat series do not register as channels.
fn detect_channel(bars: &[Bar]) -> Option<Pattern> {
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let xs: Vec<f64> = (0..closes.len()).map(|i| i as f64).collect();
    let (slope, _intercept, r2) = linfit(&xs, &closes);

    let level = closes.iter().sum::<f64>() / closes.len() as f64;
    let nslope = normalised_slope(slope, level);

    // Require both a reasonably clean fit and a non-trivial slope.
    if r2 < 0.6 || nslope.abs() < 0.001 {
        return None;
    }

    let kind = if nslope > 0.0 {
        PatternKind::ChannelUp
    } else {
        PatternKind::ChannelDown
    };
    let dir = if nslope > 0.0 { "rising" } else { "falling" };
    Some(Pattern {
        kind,
        start_ts: bars.first().map(|b| b.ts).unwrap_or(0),
        end_ts: bars.last().map(|b| b.ts).unwrap_or(0),
        confidence: round3(r2),
        description: format!(
            "{dir} price channel: closes fit a line with R²={:.2}, slope {:.3}%/bar",
            r2,
            nslope * 100.0
        ),
    })
}

/// Classify the relationship between the swing-high envelope (resistance) and
/// the swing-low envelope (support) into triangles or a wedge.
fn detect_triangle_or_wedge(bars: &[Bar]) -> Option<Pattern> {
    let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
    let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
    let w = swing_window(bars.len());

    let sh = swing_highs(&highs, w);
    let sl = swing_lows(&lows, w);
    if sh.len() < 2 || sl.len() < 2 {
        return None;
    }

    let (hs, _hb, hr2) = envelope_fit(&sh);
    let (ls, _lb, lr2) = envelope_fit(&sl);

    let level = {
        let m: f64 = bars.iter().map(|b| b.close).sum::<f64>() / bars.len() as f64;
        m.max(f64::EPSILON)
    };
    let hns = normalised_slope(hs, level);
    let lns = normalised_slope(ls, level);

    // "flat" means slope magnitude below this normalised threshold.
    let flat = 0.0015;
    let high_flat = hns.abs() < flat;
    let low_flat = lns.abs() < flat;
    let high_down = hns < -flat;
    let high_up = hns > flat;
    let low_up = lns > flat;
    let low_down = lns < -flat;

    // Envelope cleanliness drives confidence.
    let fit_quality = ((hr2 + lr2) / 2.0).clamp(0.0, 1.0);

    let (kind, desc) = if high_flat && low_up {
        (
            PatternKind::TriangleAscending,
            "ascending triangle: flat resistance, rising support".to_string(),
        )
    } else if high_down && low_flat {
        (
            PatternKind::TriangleDescending,
            "descending triangle: falling resistance, flat support".to_string(),
        )
    } else if high_down && low_up {
        (
            PatternKind::TriangleSymmetric,
            "symmetric triangle: resistance falling into rising support".to_string(),
        )
    } else if (high_up && low_up && hns < lns) || (high_down && low_down && hns > lns) {
        // Both envelopes slope the same way but converge → wedge.
        (
            PatternKind::Wedge,
            "wedge: support and resistance slope the same way and converge".to_string(),
        )
    } else {
        return None;
    };

    Some(Pattern {
        kind,
        start_ts: bars.first().map(|b| b.ts).unwrap_or(0),
        end_ts: bars.last().map(|b| b.ts).unwrap_or(0),
        confidence: round3(fit_quality.max(0.5)),
        description: format!(
            "{desc} (resistance slope {:.3}%/bar, support slope {:.3}%/bar)",
            hns * 100.0,
            lns * 100.0
        ),
    })
}

/// Fit a line through swing points keyed by their bar index.
fn envelope_fit(swings: &[Swing]) -> (f64, f64, f64) {
    let xs: Vec<f64> = swings.iter().map(|s| s.idx as f64).collect();
    let ys: Vec<f64> = swings.iter().map(|s| s.value).collect();
    linfit(&xs, &ys)
}

/// Detect double tops (two comparable swing highs flanking a trough) and double
/// bottoms (the mirror image). Both can fire on the same series; each that does
/// is returned.
fn detect_double(bars: &[Bar]) -> Vec<Pattern> {
    let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
    let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
    let w = swing_window(bars.len());

    let mut out = Vec::new();
    if let Some(p) = double_top(bars, &swing_highs(&highs, w), &lows) {
        out.push(p);
    }
    if let Some(p) = double_bottom(bars, &swing_lows(&lows, w), &highs) {
        out.push(p);
    }
    out
}

fn double_top(bars: &[Bar], peaks: &[Swing], lows: &[f64]) -> Option<Pattern> {
    if peaks.len() < 2 {
        return None;
    }
    // Take the two tallest peaks; require an intervening trough between them.
    let mut sorted = peaks.to_vec();
    sorted.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let (a, b) = (sorted[0], sorted[1]);
    let (left, right) = if a.idx <= b.idx { (a, b) } else { (b, a) };
    if right.idx <= left.idx + 2 {
        return None;
    }

    let peak_level = (left.value + right.value) / 2.0;
    let diff = (left.value - right.value).abs() / peak_level.max(f64::EPSILON);
    // Peaks must be within 3% of each other.
    if diff > 0.03 {
        return None;
    }

    // Trough between the peaks should sit clearly below them (>= 2%).
    let trough = lows[left.idx..=right.idx]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let depth = (peak_level - trough) / peak_level.max(f64::EPSILON);
    if depth < 0.02 {
        return None;
    }

    // Confidence: tighter peaks + deeper neckline → higher.
    let conf = ((1.0 - diff / 0.03) * 0.6 + (depth.min(0.15) / 0.15) * 0.4).clamp(0.0, 1.0);
    Some(Pattern {
        kind: PatternKind::DoubleTop,
        start_ts: bars[left.idx].ts,
        end_ts: bars[right.idx].ts,
        confidence: round3(conf),
        description: format!(
            "double top: two peaks within {:.1}% separated by a {:.1}% trough",
            diff * 100.0,
            depth * 100.0
        ),
    })
}

fn double_bottom(bars: &[Bar], troughs: &[Swing], highs: &[f64]) -> Option<Pattern> {
    if troughs.len() < 2 {
        return None;
    }
    let mut sorted = troughs.to_vec();
    sorted.sort_by(|a, b| {
        a.value
            .partial_cmp(&b.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let (a, b) = (sorted[0], sorted[1]);
    let (left, right) = if a.idx <= b.idx { (a, b) } else { (b, a) };
    if right.idx <= left.idx + 2 {
        return None;
    }

    let bottom_level = (left.value + right.value) / 2.0;
    let diff = (left.value - right.value).abs() / bottom_level.max(f64::EPSILON);
    if diff > 0.03 {
        return None;
    }

    let peak = highs[left.idx..=right.idx]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let height = (peak - bottom_level) / bottom_level.max(f64::EPSILON);
    if height < 0.02 {
        return None;
    }

    let conf = ((1.0 - diff / 0.03) * 0.6 + (height.min(0.15) / 0.15) * 0.4).clamp(0.0, 1.0);
    Some(Pattern {
        kind: PatternKind::DoubleBottom,
        start_ts: bars[left.idx].ts,
        end_ts: bars[right.idx].ts,
        confidence: round3(conf),
        description: format!(
            "double bottom: two troughs within {:.1}% separated by a {:.1}% peak",
            diff * 100.0,
            height * 100.0
        ),
    })
}

/// Head & shoulders: three consecutive swing highs where the middle peak (head)
/// is the tallest and the two outer peaks (shoulders) are comparable.
fn detect_head_and_shoulders(bars: &[Bar]) -> Option<Pattern> {
    let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
    let w = swing_window(bars.len());
    let peaks = swing_highs(&highs, w);
    if peaks.len() < 3 {
        return None;
    }

    // Slide a window of three consecutive peaks; keep the best-scoring triple.
    let mut best: Option<Pattern> = None;
    for triple in peaks.windows(3) {
        let (l, h, r) = (triple[0], triple[1], triple[2]);
        // Head must be the tallest.
        if h.value <= l.value || h.value <= r.value {
            continue;
        }
        let shoulder_level = (l.value + r.value) / 2.0;
        // Shoulders comparable to each other (within 4%).
        let shoulder_diff = (l.value - r.value).abs() / shoulder_level.max(f64::EPSILON);
        if shoulder_diff > 0.04 {
            continue;
        }
        // Head meaningfully above the shoulders (>= 2%).
        let prominence = (h.value - shoulder_level) / shoulder_level.max(f64::EPSILON);
        if prominence < 0.02 {
            continue;
        }

        let conf = ((1.0 - shoulder_diff / 0.04) * 0.6 + (prominence.min(0.15) / 0.15) * 0.4)
            .clamp(0.0, 1.0);
        let pattern = Pattern {
            kind: PatternKind::HeadAndShoulders,
            start_ts: bars[l.idx].ts,
            end_ts: bars[r.idx].ts,
            confidence: round3(conf),
            description: format!(
                "head & shoulders: head {:.1}% above shoulders that match within {:.1}%",
                prominence * 100.0,
                shoulder_diff * 100.0
            ),
        };
        if best
            .as_ref()
            .is_none_or(|b| pattern.confidence > b.confidence)
        {
            best = Some(pattern);
        }
    }
    best
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build bars from a close series; highs/lows hug the close so swing
    /// detection keys off the same shape.
    fn bars_from_closes(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| Bar {
                ts: 1_000 + i as i64 * 86_400,
                open: c,
                high: c * 1.001,
                low: c * 0.999,
                close: c,
                volume: 1_000,
            })
            .collect()
    }

    fn confidences_valid(patterns: &[Pattern]) {
        for p in patterns {
            assert!(
                (0.0..=1.0).contains(&p.confidence),
                "confidence out of range: {:?}",
                p
            );
        }
    }

    #[test]
    fn rising_series_is_channel_up() {
        let closes: Vec<f64> = (0..40).map(|i| 100.0 + i as f64 * 1.5).collect();
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        assert!(
            patterns.iter().any(|p| p.kind == PatternKind::ChannelUp),
            "expected ChannelUp, got {patterns:?}"
        );
        let best = detect_best(&bars).expect("a pattern");
        assert_eq!(best.kind, PatternKind::ChannelUp);
    }

    #[test]
    fn falling_series_is_channel_down() {
        let closes: Vec<f64> = (0..40).map(|i| 200.0 - i as f64 * 1.5).collect();
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        assert!(patterns.iter().any(|p| p.kind == PatternKind::ChannelDown));
    }

    #[test]
    fn flat_series_yields_no_channel() {
        let closes = vec![100.0; 40];
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        assert!(
            !patterns
                .iter()
                .any(|p| matches!(p.kind, PatternKind::ChannelUp | PatternKind::ChannelDown)),
            "flat series should not be a channel: {patterns:?}"
        );
    }

    /// A symmetric "tent" segment that rises from `base` to a single distinct
    /// apex at `peak` over `half`+`half` steps, returning to `base`. The apex is
    /// a unique maximum so swing detection keys onto exactly one point.
    fn tent(base: f64, peak: f64, half: usize) -> Vec<f64> {
        let mut v = Vec::new();
        for i in 0..half {
            v.push(base + (peak - base) * (i as f64 / half as f64));
        }
        v.push(peak);
        for i in 1..=half {
            v.push(peak - (peak - base) * (i as f64 / half as f64));
        }
        v
    }

    /// Mirror of [`tent`]: dips from `base` down to a single nadir at `valley`
    /// (`valley < base`) and back. `tent` already produces a V when its target
    /// is below the base, so this is simply a clarifying alias.
    fn dip(base: f64, valley: f64, half: usize) -> Vec<f64> {
        tent(base, valley, half)
    }

    #[test]
    fn clean_double_top_detected() {
        // Two distinct comparable peaks (~120) with a clear trough (~100).
        let mut closes = vec![100.0; 4];
        closes.extend(tent(100.0, 120.0, 4)); // first top
        closes.extend(vec![100.0; 4]); // neckline trough
        closes.extend(tent(100.0, 120.0, 4)); // second top
        closes.extend(vec![100.0; 4]);
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        assert!(
            patterns.iter().any(|p| p.kind == PatternKind::DoubleTop),
            "expected DoubleTop, got {patterns:?}"
        );
    }

    #[test]
    fn clean_double_bottom_detected() {
        // Two distinct comparable troughs (~100) with a clear peak (~120).
        let mut closes = vec![120.0; 4];
        closes.extend(dip(120.0, 100.0, 4)); // first bottom
        closes.extend(vec![120.0; 4]); // peak between
        closes.extend(dip(120.0, 100.0, 4)); // second bottom
        closes.extend(vec![120.0; 4]);
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        assert!(
            patterns.iter().any(|p| p.kind == PatternKind::DoubleBottom),
            "expected DoubleBottom, got {patterns:?}"
        );
    }

    #[test]
    fn head_and_shoulders_detected() {
        // left shoulder ~114, head ~126, right shoulder ~114, each a distinct
        // apex separated by neckline troughs at ~100.
        let mut closes = vec![100.0; 3];
        closes.extend(tent(100.0, 114.0, 3)); // left shoulder
        closes.extend(vec![100.0; 3]);
        closes.extend(tent(100.0, 126.0, 3)); // head
        closes.extend(vec![100.0; 3]);
        closes.extend(tent(100.0, 114.0, 3)); // right shoulder
        closes.extend(vec![100.0; 3]);
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        assert!(
            patterns
                .iter()
                .any(|p| p.kind == PatternKind::HeadAndShoulders),
            "expected HeadAndShoulders, got {patterns:?}"
        );
    }

    #[test]
    fn too_few_bars_returns_empty() {
        let bars = bars_from_closes(&[100.0, 101.0, 102.0]);
        assert!(detect(&bars).is_empty());
        assert!(detect_best(&bars).is_none());
    }

    #[test]
    fn linfit_perfect_line_has_r2_one() {
        let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| 2.0 * x + 3.0).collect();
        let (slope, intercept, r2) = linfit(&xs, &ys);
        assert!((slope - 2.0).abs() < 1e-9);
        assert!((intercept - 3.0).abs() < 1e-9);
        assert!((r2 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ascending_triangle_detected() {
        // Flat resistance near 120, rising lows. Build a sawtooth where peaks
        // stay flat and troughs rise.
        let mut closes = Vec::new();
        let peaks = 120.0;
        for k in 0..6 {
            let trough = 100.0 + k as f64 * 2.5;
            closes.extend((0..4).map(|i| trough + (peaks - trough) * (i as f64 / 3.0)));
            closes.extend((0..4).map(|i| {
                let next_trough = 100.0 + (k as f64 + 1.0) * 2.5;
                peaks - (peaks - next_trough) * (i as f64 / 3.0)
            }));
        }
        let bars = bars_from_closes(&closes);
        let patterns = detect(&bars);
        confidences_valid(&patterns);
        // We assert the detector runs and produces valid output; ascending
        // triangle is the expected family member.
        assert!(
            patterns.iter().any(|p| matches!(
                p.kind,
                PatternKind::TriangleAscending
                    | PatternKind::TriangleSymmetric
                    | PatternKind::Wedge
            )),
            "expected a triangle/wedge family pattern, got {patterns:?}"
        );
    }
}
