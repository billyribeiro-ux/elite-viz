//! Postgres-backed repository. Queries are runtime-checked (no `query!` macro),
//! so this compiles without a live database connection.

use anyhow::Result;
use finviz_types::{Fundamentals, Instrument, Quote, ScreenerRow};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

/// The `SELECT` that joins instruments + quotes + fundamentals + the extended
/// `screener_metrics` table into the full flat screener-row surface.
///
/// A `LEFT JOIN` on `screener_metrics` keeps the query robust to partial data:
/// an instrument with no metrics row still yields a `ScreenerRow` (the extended
/// columns map to their defaults via [`map_screener_row`]).
const SCREENER_ROWS_SQL: &str = "\
SELECT i.symbol, i.name, i.sector, i.industry, i.exchange, \
       q.price, q.change, q.change_pct, q.volume, \
       f.market_cap, f.pe, f.eps, f.dividend_yield, f.beta, \
       m.country, m.target_price, m.avg_volume, m.rel_volume, m.float_shares, m.recom, \
       m.forward_pe, m.peg, m.ps, m.pb, m.price_to_fcf, \
       m.roa, m.roe, m.roic, m.gross_margin, m.oper_margin, m.profit_margin, m.payout_ratio, \
       m.current_ratio, m.quick_ratio, m.debt_equity, m.lt_debt_equity, \
       m.insider_own, m.inst_own, m.short_float, m.short_ratio, \
       m.perf_week, m.perf_month, m.perf_quarter, m.perf_half, m.perf_year, m.perf_ytd, \
       m.volatility_w, m.volatility_m, m.rsi14, m.atr, \
       m.sma20_rel, m.sma50_rel, m.sma200_rel, m.high_52w_pct, m.low_52w_pct \
FROM instruments i \
JOIN quotes q ON q.symbol = i.symbol \
JOIN fundamentals f ON f.symbol = i.symbol \
LEFT JOIN screener_metrics m ON m.symbol = i.symbol \
ORDER BY i.symbol";

/// The upsert that materializes one `screener_metrics` row (see [`Db::seed_demo`]).
const UPSERT_METRICS_SQL: &str = "\
INSERT INTO screener_metrics (\
    symbol, country, target_price, avg_volume, rel_volume, float_shares, recom, \
    forward_pe, peg, ps, pb, price_to_fcf, \
    roa, roe, roic, gross_margin, oper_margin, profit_margin, payout_ratio, \
    current_ratio, quick_ratio, debt_equity, lt_debt_equity, \
    insider_own, inst_own, short_float, short_ratio, \
    perf_week, perf_month, perf_quarter, perf_half, perf_year, perf_ytd, \
    volatility_w, volatility_m, rsi14, atr, \
    sma20_rel, sma50_rel, sma200_rel, high_52w_pct, low_52w_pct\
) VALUES (\
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, \
    $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, \
    $33, $34, $35, $36, $37, $38, $39, $40, $41, $42\
) ON CONFLICT (symbol) DO UPDATE SET \
    country = EXCLUDED.country, target_price = EXCLUDED.target_price, \
    avg_volume = EXCLUDED.avg_volume, rel_volume = EXCLUDED.rel_volume, \
    float_shares = EXCLUDED.float_shares, recom = EXCLUDED.recom, \
    forward_pe = EXCLUDED.forward_pe, peg = EXCLUDED.peg, ps = EXCLUDED.ps, \
    pb = EXCLUDED.pb, price_to_fcf = EXCLUDED.price_to_fcf, \
    roa = EXCLUDED.roa, roe = EXCLUDED.roe, roic = EXCLUDED.roic, \
    gross_margin = EXCLUDED.gross_margin, oper_margin = EXCLUDED.oper_margin, \
    profit_margin = EXCLUDED.profit_margin, payout_ratio = EXCLUDED.payout_ratio, \
    current_ratio = EXCLUDED.current_ratio, quick_ratio = EXCLUDED.quick_ratio, \
    debt_equity = EXCLUDED.debt_equity, lt_debt_equity = EXCLUDED.lt_debt_equity, \
    insider_own = EXCLUDED.insider_own, inst_own = EXCLUDED.inst_own, \
    short_float = EXCLUDED.short_float, short_ratio = EXCLUDED.short_ratio, \
    perf_week = EXCLUDED.perf_week, perf_month = EXCLUDED.perf_month, \
    perf_quarter = EXCLUDED.perf_quarter, perf_half = EXCLUDED.perf_half, \
    perf_year = EXCLUDED.perf_year, perf_ytd = EXCLUDED.perf_ytd, \
    volatility_w = EXCLUDED.volatility_w, volatility_m = EXCLUDED.volatility_m, \
    rsi14 = EXCLUDED.rsi14, atr = EXCLUDED.atr, \
    sma20_rel = EXCLUDED.sma20_rel, sma50_rel = EXCLUDED.sma50_rel, \
    sma200_rel = EXCLUDED.sma200_rel, high_52w_pct = EXCLUDED.high_52w_pct, \
    low_52w_pct = EXCLUDED.low_52w_pct";

/// Map one joined `instruments+quotes+fundamentals+screener_metrics` row to a
/// full [`ScreenerRow`].
///
/// Extended (`screener_metrics`) columns are read with `try_get`, falling back
/// to the type's natural default when the column is `NULL` or the row is
/// absent (the `LEFT JOIN` produces all-`NULL` metric columns). This keeps the
/// mapping robust to partial data: a freshly inserted instrument with no
/// metrics row still produces a coherent `ScreenerRow`.
#[cfg(feature = "postgres")]
fn map_screener_row<R: Row>(r: &R) -> ScreenerRow
where
    for<'a> &'a str: sqlx::ColumnIndex<R>,
    for<'a> String: sqlx::Decode<'a, R::Database> + sqlx::Type<R::Database>,
    for<'a> i64: sqlx::Decode<'a, R::Database> + sqlx::Type<R::Database>,
    for<'a> f64: sqlx::Decode<'a, R::Database> + sqlx::Type<R::Database>,
    for<'a> Option<f64>: sqlx::Decode<'a, R::Database> + sqlx::Type<R::Database>,
{
    // f64 column, NULL or missing -> 0.0
    let f = |col: &str| -> f64 { r.try_get(col).unwrap_or(0.0) };
    // Option<f64> column, missing -> None (NULL already decodes to None)
    let of = |col: &str| -> Option<f64> { r.try_get(col).unwrap_or(None) };

    ScreenerRow {
        // identity / descriptive
        symbol: r.get("symbol"),
        name: r.get("name"),
        sector: r.get("sector"),
        industry: r.get("industry"),
        exchange: r.get("exchange"),
        country: r.try_get("country").unwrap_or_default(),
        target_price: of("target_price"),
        avg_volume: f("avg_volume"),
        rel_volume: f("rel_volume"),
        float_shares: f("float_shares"),
        recom: of("recom"),

        // market / quote
        price: r.get("price"),
        change: r.get("change"),
        change_pct: r.get("change_pct"),
        volume: r.get("volume"),

        // valuation
        market_cap: r.get("market_cap"),
        pe: r.get("pe"),
        forward_pe: of("forward_pe"),
        peg: of("peg"),
        ps: of("ps"),
        pb: of("pb"),
        price_to_fcf: of("price_to_fcf"),
        eps: r.get("eps"),
        dividend_yield: r.get("dividend_yield"),
        beta: r.get("beta"),

        // profitability
        roa: of("roa"),
        roe: of("roe"),
        roic: of("roic"),
        gross_margin: of("gross_margin"),
        oper_margin: of("oper_margin"),
        profit_margin: of("profit_margin"),
        payout_ratio: of("payout_ratio"),

        // financial health
        current_ratio: of("current_ratio"),
        quick_ratio: of("quick_ratio"),
        debt_equity: of("debt_equity"),
        lt_debt_equity: of("lt_debt_equity"),

        // ownership
        insider_own: of("insider_own"),
        inst_own: of("inst_own"),
        short_float: of("short_float"),
        short_ratio: of("short_ratio"),

        // performance
        perf_week: f("perf_week"),
        perf_month: f("perf_month"),
        perf_quarter: f("perf_quarter"),
        perf_half: f("perf_half"),
        perf_year: f("perf_year"),
        perf_ytd: f("perf_ytd"),

        // technical
        volatility_w: f("volatility_w"),
        volatility_m: f("volatility_m"),
        rsi14: f("rsi14"),
        atr: f("atr"),
        sma20_rel: f("sma20_rel"),
        sma50_rel: f("sma50_rel"),
        sma200_rel: f("sma200_rel"),
        high_52w_pct: f("high_52w_pct"),
        low_52w_pct: f("low_52w_pct"),
    }
}

/// A connection pool plus the queries the API needs.
#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Connect and build a bounded connection pool.
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Run pending schema migrations from `./migrations`.
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn instruments(&self) -> Result<Vec<Instrument>> {
        let rows = sqlx::query(
            "SELECT symbol, name, sector, industry, exchange FROM instruments ORDER BY symbol",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| Instrument {
                symbol: r.get("symbol"),
                name: r.get("name"),
                sector: r.get("sector"),
                industry: r.get("industry"),
                exchange: r.get("exchange"),
            })
            .collect())
    }

    pub async fn quote(&self, symbol: &str) -> Result<Option<Quote>> {
        let row = sqlx::query(
            "SELECT symbol, price, change, change_pct, volume, prev_close, day_high, day_low, ts \
             FROM quotes WHERE symbol = $1",
        )
        .bind(symbol.to_uppercase())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Quote {
            symbol: r.get("symbol"),
            price: r.get("price"),
            change: r.get("change"),
            change_pct: r.get("change_pct"),
            volume: r.get("volume"),
            prev_close: r.get("prev_close"),
            day_high: r.get("day_high"),
            day_low: r.get("day_low"),
            ts: r.get("ts"),
        }))
    }

    pub async fn fundamentals(&self, symbol: &str) -> Result<Option<Fundamentals>> {
        let row = sqlx::query(
            "SELECT symbol, market_cap, pe, eps, dividend_yield, beta, shares_outstanding \
             FROM fundamentals WHERE symbol = $1",
        )
        .bind(symbol.to_uppercase())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Fundamentals {
            symbol: r.get("symbol"),
            market_cap: r.get("market_cap"),
            pe: r.get("pe"),
            eps: r.get("eps"),
            dividend_yield: r.get("dividend_yield"),
            beta: r.get("beta"),
            shares_outstanding: r.get("shares_outstanding"),
        }))
    }

    /// Join instruments + quotes + fundamentals + extended `screener_metrics`
    /// into flat screener rows that carry the full FINVIZ-style metric surface.
    ///
    /// The metrics table is `LEFT JOIN`ed, so an instrument without a metrics
    /// row still produces a complete [`ScreenerRow`] (extended fields default).
    pub async fn screener_rows(&self) -> Result<Vec<ScreenerRow>> {
        let rows = sqlx::query(SCREENER_ROWS_SQL).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(map_screener_row).collect())
    }

    /// Populate `instruments`, `quotes`, `fundamentals`, and `screener_metrics`
    /// from a flat set of [`ScreenerRow`]s, so a fresh database reaches parity
    /// with the in-memory seed dataset.
    ///
    /// Callers pass the rows in (rather than this crate reaching into
    /// `finviz-core`), which keeps the dependency graph acyclic and one-way
    /// (`finviz-db` depends only on `finviz-types`). To fill a DB from the
    /// canonical seed, map `finviz_core::seed::dataset(now)` into `ScreenerRow`s
    /// at the call site and hand them here.
    ///
    /// Every table uses `INSERT ... ON CONFLICT DO UPDATE` (upsert), so the call
    /// is idempotent and safe to re-run. All writes share one transaction: the
    /// whole seed commits or rolls back together. `prev_close`, `day_high`, and
    /// `day_low` are reconstructed from the quote fields the screener row carries.
    pub async fn seed_demo(&self, rows: &[ScreenerRow]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for row in rows {
            let symbol = row.symbol.to_uppercase();
            let prev_close = row.price - row.change;

            sqlx::query(
                "INSERT INTO instruments (symbol, name, sector, industry, exchange) \
                 VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (symbol) DO UPDATE SET \
                     name = EXCLUDED.name, sector = EXCLUDED.sector, \
                     industry = EXCLUDED.industry, exchange = EXCLUDED.exchange",
            )
            .bind(&symbol)
            .bind(&row.name)
            .bind(&row.sector)
            .bind(&row.industry)
            .bind(&row.exchange)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO quotes (symbol, price, change, change_pct, volume, prev_close, day_high, day_low, ts) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                 ON CONFLICT (symbol) DO UPDATE SET \
                     price = EXCLUDED.price, change = EXCLUDED.change, \
                     change_pct = EXCLUDED.change_pct, volume = EXCLUDED.volume, \
                     prev_close = EXCLUDED.prev_close, day_high = EXCLUDED.day_high, \
                     day_low = EXCLUDED.day_low, ts = EXCLUDED.ts",
            )
            .bind(&symbol)
            .bind(row.price)
            .bind(row.change)
            .bind(row.change_pct)
            .bind(row.volume)
            .bind(prev_close)
            .bind(row.price.max(prev_close) * 1.012)
            .bind(row.price.min(prev_close) * 0.988)
            .bind(0_i64)
            .execute(&mut *tx)
            .await?;

            let shares = if row.price > 0.0 {
                row.market_cap / row.price
            } else {
                0.0
            };
            sqlx::query(
                "INSERT INTO fundamentals (symbol, market_cap, pe, eps, dividend_yield, beta, shares_outstanding) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (symbol) DO UPDATE SET \
                     market_cap = EXCLUDED.market_cap, pe = EXCLUDED.pe, eps = EXCLUDED.eps, \
                     dividend_yield = EXCLUDED.dividend_yield, beta = EXCLUDED.beta, \
                     shares_outstanding = EXCLUDED.shares_outstanding",
            )
            .bind(&symbol)
            .bind(row.market_cap)
            .bind(row.pe)
            .bind(row.eps)
            .bind(row.dividend_yield)
            .bind(row.beta)
            .bind(shares)
            .execute(&mut *tx)
            .await?;

            sqlx::query(UPSERT_METRICS_SQL)
                .bind(&symbol)
                .bind(&row.country)
                .bind(row.target_price)
                .bind(row.avg_volume)
                .bind(row.rel_volume)
                .bind(row.float_shares)
                .bind(row.recom)
                .bind(row.forward_pe)
                .bind(row.peg)
                .bind(row.ps)
                .bind(row.pb)
                .bind(row.price_to_fcf)
                .bind(row.roa)
                .bind(row.roe)
                .bind(row.roic)
                .bind(row.gross_margin)
                .bind(row.oper_margin)
                .bind(row.profit_margin)
                .bind(row.payout_ratio)
                .bind(row.current_ratio)
                .bind(row.quick_ratio)
                .bind(row.debt_equity)
                .bind(row.lt_debt_equity)
                .bind(row.insider_own)
                .bind(row.inst_own)
                .bind(row.short_float)
                .bind(row.short_ratio)
                .bind(row.perf_week)
                .bind(row.perf_month)
                .bind(row.perf_quarter)
                .bind(row.perf_half)
                .bind(row.perf_year)
                .bind(row.perf_ytd)
                .bind(row.volatility_w)
                .bind(row.volatility_m)
                .bind(row.rsi14)
                .bind(row.atr)
                .bind(row.sma20_rel)
                .bind(row.sma50_rel)
                .bind(row.sma200_rel)
                .bind(row.high_52w_pct)
                .bind(row.low_52w_pct)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every extended `screener_metrics` column the `ScreenerRow` surface needs
    /// must be selected by the join and accepted by the upsert. This guards
    /// against the two SQL strings drifting out of sync with each other (and
    /// with the migration) without requiring a live database.
    const EXTENDED_COLS: &[&str] = &[
        "country",
        "target_price",
        "avg_volume",
        "rel_volume",
        "float_shares",
        "recom",
        "forward_pe",
        "peg",
        "ps",
        "pb",
        "price_to_fcf",
        "roa",
        "roe",
        "roic",
        "gross_margin",
        "oper_margin",
        "profit_margin",
        "payout_ratio",
        "current_ratio",
        "quick_ratio",
        "debt_equity",
        "lt_debt_equity",
        "insider_own",
        "inst_own",
        "short_float",
        "short_ratio",
        "perf_week",
        "perf_month",
        "perf_quarter",
        "perf_half",
        "perf_year",
        "perf_ytd",
        "volatility_w",
        "volatility_m",
        "rsi14",
        "atr",
        "sma20_rel",
        "sma50_rel",
        "sma200_rel",
        "high_52w_pct",
        "low_52w_pct",
    ];

    #[test]
    fn screener_select_reads_every_extended_column() {
        for col in EXTENDED_COLS {
            assert!(
                SCREENER_ROWS_SQL.contains(&format!("m.{col}")),
                "screener SELECT is missing extended column `{col}`"
            );
        }
        // Core columns + the LEFT JOIN onto the metrics table must be present.
        assert!(SCREENER_ROWS_SQL.contains("LEFT JOIN screener_metrics m"));
        assert!(SCREENER_ROWS_SQL.contains("f.market_cap"));
    }

    #[test]
    fn upsert_writes_every_extended_column_and_is_idempotent() {
        for col in EXTENDED_COLS {
            assert!(
                UPSERT_METRICS_SQL.contains(col),
                "metrics upsert is missing column `{col}`"
            );
            // ON CONFLICT path must refresh each extended column.
            assert!(
                UPSERT_METRICS_SQL.contains(&format!("{col} = EXCLUDED.{col}")),
                "metrics upsert does not update column `{col}` on conflict"
            );
        }
        assert!(UPSERT_METRICS_SQL.contains("ON CONFLICT (symbol) DO UPDATE"));
    }

    #[test]
    fn upsert_bind_count_matches_placeholders() {
        // 1 (symbol) + 41 extended columns = 42 bind parameters.
        assert_eq!(EXTENDED_COLS.len() + 1, 42);
        assert!(UPSERT_METRICS_SQL.contains("$42"));
        assert!(!UPSERT_METRICS_SQL.contains("$43"));
    }
}
