use crate::endpoints::models::custodian_models as models;
use crate::endpoints::request_json_validator::ValidJson;
use crate::endpoints::response_utils::*;
use crate::repositories::errors::DatabaseError;
use crate::services::custodian_service::CreateError;
use crate::state::AppState;
use crate::utils::auth_middleware::Session;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Extension;

pub async fn create(
    State(state): State<AppState>,
    Extension(session): Session,
    ValidJson(request): ValidJson<models::create::Request>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&request.0) {
        return *response;
    }

    match request.into_entity(session.user_id) {
        Ok(entity) => match state.custodian_service.create(entity).await {
            Ok(new_id) => response_created_new_id(new_id),
            Err(e) => match e {
                CreateError::NameAlreadyExists => response_duplicated_value("Name"),
                CreateError::DatabaseError(e) => response_error(&e.to_string()),
            },
        },
        Err(e) => response_bad_request(&e),
    }
}

pub async fn single(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.custodian_service.single(id, &session.user_id).await {
        Ok(item) => response_ok(item),
        Err(e) => response_error(&e.to_string()),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
    ValidJson(request): ValidJson<models::update::Request>,
) -> impl IntoResponse {
    if session.user_id.is_empty() {
        response_unhautorized("User ID is empty")
    } else {
        if let Err(response) = validate_request(&request.0) {
            return *response;
        }

        match request.into_entity(id, session.user_id) {
            Ok(entity) => match state.custodian_service.update(entity).await {
                Ok(()) => response_ok("Custodian updated successfully"),
                Err(e) => response_error(&e.to_string()),
            },
            Err(e) => response_bad_request(&e),
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.custodian_service.delete(id, &session.user_id).await {
        Ok(()) => response_ok(()),
        Err(e)
            if matches!(
                e,
                DatabaseError::RecordNotFound | DatabaseError::RecordNotFoundWithId(_)
            ) =>
        {
            response_not_found(&e.to_string())
        }
        Err(e) => response_error(&e.to_string()),
    }
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match state.custodian_service.list().await {
        // no need to convert to a model
        Ok(entities) => response_ok(entities),
        Err(e) => response_error(&e.to_string()),
    }
}
