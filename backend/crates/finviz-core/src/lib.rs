//! Core runtime for the FINVIZ Elite+ backend: configuration and the shared,
//! in-memory market dataset (the storage-backed implementation lands later).

pub mod config;
pub mod news;
pub mod seed;
pub mod state;

pub use config::Config;
pub use state::AppState;
