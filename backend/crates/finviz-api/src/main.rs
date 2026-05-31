//! FINVIZ Elite+ API server (Axum).
//!
//! Serves a versioned REST + WebSocket API over an in-memory market dataset:
//! market data, the screener DSL, technical indicators, watchlists, portfolio
//! valuation, alerts, JWT-based auth, and pluggable live-quote providers — all
//! behind CORS, request tracing, and graceful shutdown. An optional
//! PostgreSQL-backed store lives in `finviz-db` (the `postgres` feature).

mod auth;
mod error;
mod providers;
mod routes;

use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum::{Json, Router};
use finviz_core::{AppState, Config};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,finviz_api=debug")),
        )
        .init();

    let config = Config::from_env();
    let state = AppState::seeded();

    let app = build_app(&config, state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "FINVIZ Elite+ API listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn build_app(config: &Config, state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ws/quotes", get(routes::ws::quotes))
        .route("/ws/screener-updates", get(routes::ws::screener_updates))
        .nest("/api/v1", routes::api_router())
        .layer(cors_layer(config))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "finviz-api", "version": env!("CARGO_PKG_VERSION") }))
}

fn cors_layer(config: &Config) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    if config.cors_origin == "*" {
        base.allow_origin(Any)
    } else {
        match config.cors_origin.parse::<HeaderValue>() {
            Ok(origin) => base.allow_origin(origin),
            Err(_) => {
                tracing::warn!(origin = %config.cors_origin, "invalid CORS origin; allowing any");
                base.allow_origin(Any)
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
