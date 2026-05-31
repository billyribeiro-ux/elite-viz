//! `/api/v1/screener/saved/*` — CRUD over user-saved screener queries.
//!
//! On create and update the `query` is validated by parsing it with
//! [`finviz_screener::parse`]; an unparseable query is rejected with `400`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use finviz_core::AppState;
use finviz_types::SavedScreen;
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

pub async fn list(State(state): State<AppState>) -> Json<Vec<SavedScreen>> {
    Json(state.saved_screens())
}

#[derive(Debug, Deserialize)]
pub struct CreateSavedScreen {
    pub name: String,
    pub query: String,
    pub sort: Option<String>,
    pub order: Option<String>,
}

/// Reject an empty name or an unparseable query.
fn validate(name: &str, query: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("screen name is required".into()));
    }
    finviz_screener::parse(query).map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(())
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateSavedScreen>,
) -> ApiResult<(StatusCode, Json<SavedScreen>)> {
    validate(&body.name, &body.query)?;
    let screen = state.create_saved_screen(body.name, body.query, body.sort, body.order);
    Ok((StatusCode::CREATED, Json(screen)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavedScreen {
    pub name: Option<String>,
    pub query: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateSavedScreen>,
) -> ApiResult<Json<SavedScreen>> {
    if let Some(name) = &body.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("screen name cannot be empty".into()));
        }
    }
    if let Some(query) = &body.query {
        finviz_screener::parse(query).map_err(|e| AppError::BadRequest(e.to_string()))?;
    }
    state
        .update_saved_screen(
            &id,
            body.name,
            body.query,
            body.sort.map(Some),
            body.order.map(Some),
        )
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("no saved screen `{id}`")))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    if state.delete_saved_screen(&id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("no saved screen `{id}`")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_name() {
        assert!(validate("  ", "price > 10").is_err());
    }

    #[test]
    fn validate_rejects_bad_query() {
        assert!(validate("My Screen", "price >>> 10 and and").is_err());
    }

    #[test]
    fn validate_accepts_good_input() {
        assert!(validate("My Screen", "price > 100 and pe < 30").is_ok());
    }

    #[test]
    fn store_crud_round_trip() {
        let state = AppState::seeded();
        assert!(state.saved_screens().is_empty());

        let created = state.create_saved_screen(
            "Cheap".into(),
            "pe < 15".into(),
            Some("pe".into()),
            Some("asc".into()),
        );
        let list = state.saved_screens();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);
        assert_eq!(list[0].name, "Cheap");

        let updated = state
            .update_saved_screen(
                &created.id,
                Some("Cheaper".into()),
                Some("pe < 10".into()),
                None,
                Some(Some("desc".into())),
            )
            .expect("update should find the screen");
        assert_eq!(updated.name, "Cheaper");
        assert_eq!(updated.query, "pe < 10");
        assert_eq!(updated.sort.as_deref(), Some("pe")); // untouched
        assert_eq!(updated.order.as_deref(), Some("desc"));

        assert!(state.delete_saved_screen(&created.id));
        assert!(!state.delete_saved_screen(&created.id));
        assert!(state.saved_screens().is_empty());
    }

    #[test]
    fn update_missing_returns_none() {
        let state = AppState::seeded();
        assert!(state
            .update_saved_screen("nope", Some("x".into()), None, None, None)
            .is_none());
    }
}
