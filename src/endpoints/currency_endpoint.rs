use axum::{extract::Path, extract::State, response::IntoResponse, Extension};

use super::response_utils::{
    response_bad_request, response_created_new_id, response_error, response_not_found, response_ok,
};
use crate::dependency_injection::AppState;
use crate::endpoints::models::currency_models as models;
use crate::endpoints::request_json_validator::ValidJson;

use crate::endpoints::response_utils::response_ok_no_data;
use crate::utils::auth_middleware::Session;

pub async fn create(
    State(state): State<AppState>,
    ValidJson(data): ValidJson<models::CreateRequest>,
) -> impl IntoResponse {
    match data.to_entity() {
        Ok(entity) => match state.currency_service.create(entity).await {
            Ok(new_id) => response_created_new_id(new_id),
            Err(e) => response_error(&e),
        },
        Err(e) => response_bad_request(&e),
    }
}

pub async fn update(
    State(state): State<AppState>,
    ValidJson(data): ValidJson<models::UpdateRequest>,
) -> impl IntoResponse {
    match data.to_entity() {
        Ok(entity) => match state.currency_service.update(entity).await {
            Ok(()) => response_ok("Currency updated successfully"),
            Err(e) => response_error(&e),
        },
        Err(e) => response_bad_request(&e),
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    match state.currency_service.delete(id).await {
        Ok(()) => response_ok_no_data(),
        Err(e) => response_error(&e),
    }
}

pub async fn single(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    match state.currency_service.try_get(id) {
        Some(currency) => response_ok(models::Currency::from(currency)),
        None => response_not_found(&format!("Currency not found with id = {}", id)),
    }
}

pub async fn list_all(
    State(state): State<AppState>,
    Extension(_session): Session,
) -> impl IntoResponse {
    let entities = state.currency_service.all();
    let models: Vec<models::Currency> = entities
        .iter()
        .map(|e| models::Currency::from(e.clone()))
        .collect();
    response_ok(models)
}

pub async fn list_of_user(
    State(state): State<AppState>,
    Extension(session): Session,
) -> impl IntoResponse {
    match state.currency_service.list_for_user(&session.user_id).await {
        Ok(data) => response_ok(data),
        Err(err) => response_error(&err),
    }
}

pub async fn enable(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state
        .currency_service
        .enable_currency_for_user(session.user_id, id, true)
        .await
    {
        // true
        Ok(()) => response_ok_no_data(),
        Err(e) => response_error(&e),
    }
}

pub async fn disable(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state
        .currency_service
        .enable_currency_for_user(session.user_id, id, false)
        .await
    {
        // false
        Ok(()) => response_ok_no_data(),
        Err(e) => response_error(&e),
    }
}
