//! `/api/v1/settings/provider` — configure and test the live data provider.
//!
//! The API key is write-only: it is accepted on update but never returned in
//! full (only a masked hint and a "set" flag).

use axum::extract::State;
use axum::Json;
use finviz_core::AppState;
use finviz_types::{ProviderConfig, ProviderKind};
use serde::Serialize;

use crate::providers;

#[derive(Debug, Serialize)]
pub struct ProviderView {
    pub kind: ProviderKind,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub api_key_set: bool,
    /// Last 4 characters of the stored key, for confirmation in the UI.
    pub api_key_hint: Option<String>,
}

fn view(cfg: &ProviderConfig) -> ProviderView {
    let key = cfg.api_key.as_deref().unwrap_or_default();
    ProviderView {
        kind: cfg.kind,
        base_url: cfg.base_url.clone(),
        enabled: cfg.enabled,
        api_key_set: !key.is_empty(),
        api_key_hint: (key.len() >= 4).then(|| format!("••••{}", &key[key.len() - 4..])),
    }
}

pub async fn get(State(state): State<AppState>) -> Json<ProviderView> {
    Json(view(&state.provider_config()))
}

pub async fn update(
    State(state): State<AppState>,
    Json(cfg): Json<ProviderConfig>,
) -> Json<ProviderView> {
    Json(view(&state.set_provider_config(cfg)))
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub message: String,
}

/// Test a candidate config without saving it. An empty `api_key` falls back to
/// the currently-stored key so users can re-test without re-typing secrets.
pub async fn test(
    State(state): State<AppState>,
    Json(mut cfg): Json<ProviderConfig>,
) -> Json<TestResult> {
    let key_missing = cfg.api_key.as_deref().is_none_or(str::is_empty);
    if key_missing {
        cfg.api_key = state.provider_config().api_key;
    }
    match providers::health(&cfg).await {
        Ok(()) => Json(TestResult {
            ok: true,
            message: format!("Connected to {:?} provider.", cfg.kind),
        }),
        Err(e) => Json(TestResult {
            ok: false,
            message: e.to_string(),
        }),
    }
}
