//! PostgreSQL persistence for FINVIZ Elite+.
//!
//! Gated behind the `postgres` feature so the default workspace build needs no
//! database. The running API server currently defaults to the in-memory store
//! ([`finviz_core::AppState`]); this crate provides the storage-backed path and
//! the schema migrations it runs.

#[cfg(feature = "postgres")]
mod pg;

#[cfg(feature = "postgres")]
pub use pg::Db;
