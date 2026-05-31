//! `PostgreSQL` persistence for FINVIZ Elite+.
//!
//! Gated behind the `postgres` feature so the default workspace build needs no
//! database. The running API server defaults to the in-memory store
//! (`finviz_core::AppState`); this crate provides the optional storage-backed
//! path and the schema migrations it runs.
//!
//! As of Wave 10 the Postgres path materializes the **full** screener-row
//! surface, at parity with the in-memory seed. The original schema carried only
//! the ~14 core columns; migration `0002_extended_metrics.sql` adds a
//! `screener_metrics` table holding every extended FINVIZ-style field
//! (descriptive, valuation, profitability, financial health, ownership,
//! performance, technical). [`Db::screener_rows`] `LEFT JOIN`s it to return a
//! complete [`finviz_types::ScreenerRow`], and [`Db::seed_demo`] upserts a fresh
//! database to parity from a caller-supplied set of rows.
//!
//! All queries are runtime-checked (plain `sqlx::query`, never the `query!`
//! macro), so the crate compiles and is clippy-clean offline, without a live
//! database or `DATABASE_URL`.

#[cfg(feature = "postgres")]
mod pg;

#[cfg(feature = "postgres")]
pub use pg::Db;
