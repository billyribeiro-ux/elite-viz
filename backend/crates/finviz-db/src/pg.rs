//! Postgres-backed repository. Queries are runtime-checked (no `query!` macro),
//! so this compiles without a live database connection.

use anyhow::Result;
use finviz_types::{Fundamentals, Instrument, Quote, ScreenerRow};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

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

    /// Join instruments + quotes + fundamentals into flat screener rows.
    pub async fn screener_rows(&self) -> Result<Vec<ScreenerRow>> {
        let rows = sqlx::query(
            "SELECT i.symbol, i.name, i.sector, i.industry, i.exchange, \
                    q.price, q.change, q.change_pct, q.volume, \
                    f.market_cap, f.pe, f.eps, f.dividend_yield, f.beta \
             FROM instruments i \
             JOIN quotes q ON q.symbol = i.symbol \
             JOIN fundamentals f ON f.symbol = i.symbol",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| ScreenerRow {
                symbol: r.get("symbol"),
                name: r.get("name"),
                sector: r.get("sector"),
                industry: r.get("industry"),
                exchange: r.get("exchange"),
                price: r.get("price"),
                change: r.get("change"),
                change_pct: r.get("change_pct"),
                volume: r.get("volume"),
                market_cap: r.get("market_cap"),
                pe: r.get("pe"),
                eps: r.get("eps"),
                dividend_yield: r.get("dividend_yield"),
                beta: r.get("beta"),
            })
            .collect())
    }
}
