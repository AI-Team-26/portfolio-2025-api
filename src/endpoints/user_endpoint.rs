use crate::endpoints::models::user_models as models;
use crate::{
    dependency_injection::AppState,
    endpoints::{
        request_json_validator::ValidJson,
        response_utils::{response_error, response_ok_no_data},
    },
    info,
    utils::auth_middleware::Session,
};
use axum::{extract::State, response::IntoResponse, Extension};

pub async fn update(
    State(state): State<AppState>,
    Extension(session): Session,
    ValidJson(request): ValidJson<models::update::Request>,
) -> impl IntoResponse {
    // Here you would add the logic to update the user's currency in the database.
    // For now, it's a placeholder.
    info!(
        "Updating user currency, new currency: {}",
        request.currency_id
    );

    match state
        .user_service
        .update_currency(session.user_id, request.currency_id)
        .await
    {
        Ok(()) => response_ok_no_data(),
        Err(e) => response_error(&e.message),
    }
}
