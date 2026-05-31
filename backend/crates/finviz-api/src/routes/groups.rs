//! `/api/v1/groups` — aggregate the screener universe by a descriptive key.

use axum::extract::{Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::ScreenerRow;
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};

#[derive(Debug, Deserialize)]
pub struct GroupsQuery {
    by: Option<String>,
}

/// Which descriptive field to bucket rows by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKey {
    Sector,
    Industry,
    Country,
}

impl GroupKey {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sector" => Some(GroupKey::Sector),
            "industry" => Some(GroupKey::Industry),
            "country" => Some(GroupKey::Country),
            _ => None,
        }
    }

    fn of(self, row: &ScreenerRow) -> &str {
        match self {
            GroupKey::Sector => &row.sector,
            GroupKey::Industry => &row.industry,
            GroupKey::Country => &row.country,
        }
    }
}

/// One aggregated bucket. Averages are over the rows in the bucket; `avg_pe`
/// ignores rows whose P/E is `None` (and is itself `None` if none have one).
#[derive(Debug, Serialize)]
pub struct GroupRow {
    pub name: String,
    pub count: usize,
    pub avg_change_pct: f64,
    pub avg_pe: Option<f64>,
    pub total_market_cap: f64,
    pub avg_perf_week: f64,
    pub avg_perf_month: f64,
    pub avg_perf_year: f64,
}

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<GroupsQuery>,
) -> ApiResult<Json<Vec<GroupRow>>> {
    let by = q.by.as_deref().unwrap_or("sector");
    let key = GroupKey::parse(by)
        .ok_or_else(|| AppError::BadRequest(format!("unknown group key `{by}`")))?;
    Ok(Json(aggregate(&state.screener_rows(), key)))
}

/// Bucket `rows` by `key`, computing per-group counts, averages, and totals.
/// Groups are returned sorted by `total_market_cap` descending. Float outputs
/// are rounded to two decimal places.
pub fn aggregate(rows: &[ScreenerRow], key: GroupKey) -> Vec<GroupRow> {
    use std::collections::HashMap;

    #[derive(Default)]
    struct Acc {
        count: usize,
        sum_change_pct: f64,
        sum_pe: f64,
        pe_count: usize,
        total_market_cap: f64,
        sum_perf_week: f64,
        sum_perf_month: f64,
        sum_perf_year: f64,
    }

    // Preserve first-seen order for deterministic ties before the final sort.
    let mut order: Vec<String> = Vec::new();
    let mut acc: HashMap<String, Acc> = HashMap::new();

    for row in rows {
        let name = key.of(row).to_string();
        let entry = acc.entry(name.clone()).or_insert_with(|| {
            order.push(name.clone());
            Acc::default()
        });
        entry.count += 1;
        entry.sum_change_pct += row.change_pct;
        if let Some(pe) = row.pe {
            entry.sum_pe += pe;
            entry.pe_count += 1;
        }
        entry.total_market_cap += row.market_cap;
        entry.sum_perf_week += row.perf_week;
        entry.sum_perf_month += row.perf_month;
        entry.sum_perf_year += row.perf_year;
    }

    let mut out: Vec<GroupRow> = order
        .into_iter()
        .map(|name| {
            let a = &acc[&name];
            let n = a.count as f64;
            GroupRow {
                name,
                count: a.count,
                avg_change_pct: round2(a.sum_change_pct / n),
                avg_pe: (a.pe_count > 0).then(|| round2(a.sum_pe / a.pe_count as f64)),
                total_market_cap: round2(a.total_market_cap),
                avg_perf_week: round2(a.sum_perf_week / n),
                avg_perf_month: round2(a.sum_perf_month / n),
                avg_perf_year: round2(a.sum_perf_year / n),
            }
        })
        .collect();

    out.sort_by(|a, b| {
        b.total_market_cap
            .partial_cmp(&a.total_market_cap)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(sector: &str, change_pct: f64, pe: Option<f64>, mcap: f64) -> ScreenerRow {
        ScreenerRow {
            symbol: "X".into(),
            name: "X".into(),
            sector: sector.into(),
            industry: "ind".into(),
            exchange: "NASDAQ".into(),
            country: "USA".into(),
            target_price: None,
            avg_volume: 0.0,
            rel_volume: 0.0,
            float_shares: 0.0,
            recom: None,
            price: 0.0,
            change: 0.0,
            change_pct,
            volume: 0,
            market_cap: mcap,
            pe,
            forward_pe: None,
            peg: None,
            ps: None,
            pb: None,
            price_to_fcf: None,
            eps: None,
            dividend_yield: None,
            beta: None,
            roa: None,
            roe: None,
            roic: None,
            gross_margin: None,
            oper_margin: None,
            profit_margin: None,
            payout_ratio: None,
            current_ratio: None,
            quick_ratio: None,
            debt_equity: None,
            lt_debt_equity: None,
            insider_own: None,
            inst_own: None,
            short_float: None,
            short_ratio: None,
            perf_week: 1.0,
            perf_month: 2.0,
            perf_quarter: 0.0,
            perf_half: 0.0,
            perf_year: 3.0,
            perf_ytd: 0.0,
            volatility_w: 0.0,
            volatility_m: 0.0,
            rsi14: 0.0,
            atr: 0.0,
            sma20_rel: 0.0,
            sma50_rel: 0.0,
            sma200_rel: 0.0,
            high_52w_pct: 0.0,
            low_52w_pct: 0.0,
        }
    }

    #[test]
    fn aggregate_groups_counts_averages_and_sorts_by_market_cap() {
        let rows = vec![
            row("Tech", 2.0, Some(10.0), 100.0),
            row("Tech", 4.0, None, 300.0), // None P/E excluded from avg.
            row("Energy", -1.0, Some(8.0), 50.0),
        ];

        let out = aggregate(&rows, GroupKey::Sector);
        assert_eq!(out.len(), 2);

        // Sorted by total_market_cap desc => Tech (400) before Energy (50).
        let tech = &out[0];
        assert_eq!(tech.name, "Tech");
        assert_eq!(tech.count, 2);
        assert_eq!(tech.total_market_cap, 400.0);
        assert_eq!(tech.avg_change_pct, 3.0);
        // Only one Tech row had a P/E (10.0); the None is excluded.
        assert_eq!(tech.avg_pe, Some(10.0));
        assert_eq!(tech.avg_perf_week, 1.0);
        assert_eq!(tech.avg_perf_month, 2.0);
        assert_eq!(tech.avg_perf_year, 3.0);

        let energy = &out[1];
        assert_eq!(energy.name, "Energy");
        assert_eq!(energy.count, 1);
        assert_eq!(energy.avg_pe, Some(8.0));
        assert_eq!(energy.total_market_cap, 50.0);
    }

    #[test]
    fn aggregate_avg_pe_is_none_when_all_missing() {
        let rows = vec![row("Tech", 1.0, None, 10.0), row("Tech", 1.0, None, 20.0)];
        let out = aggregate(&rows, GroupKey::Sector);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].avg_pe, None);
    }
}
