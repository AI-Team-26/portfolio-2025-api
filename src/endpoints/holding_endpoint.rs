use crate::dependency_injection::AppState;
use crate::endpoints::models::holding_models as models;
use crate::endpoints::request_json_validator::ValidJson;
use crate::endpoints::request_validator::RuleNumber;
use crate::endpoints::response_utils::*;
use crate::repositories::errors::ErrorKind;
use crate::utils::auth_middleware::Session;
use crate::validate;
use axum::extract::State;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::Extension;

pub async fn create(
    State(state): State<AppState>,
    Extension(session): Session,
    ValidJson(request): ValidJson<models::create::Request>,
) -> impl IntoResponse {
    validate!(
        //"Name", request.date, RuleString::NotEmpty;
        "Amount", request.amount, RuleNumber::NotZero;
    );

    match state
        .holding_service
        .create(&session.user_id, request)
        .await
    {
        Ok(new_id) => response_created_new_id(new_id),
        Err(e) => response_error(&e),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
    ValidJson(request): ValidJson<models::update::Request>,
) -> impl IntoResponse {
    validate!(
        //"Name", request.date, RuleString::NotEmpty;
        "Amount", request.amount, RuleNumber::NotZero;
    );

    match state
        .holding_service
        .update(&session.user_id, id, request)
        .await
    {
        Ok(()) => response_ok_no_data(),
        Err(e) => response_error(&e.message),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.holding_service.delete(&session.user_id, id).await {
        Ok(()) => response_ok(()),
        Err(e) if e.kind == ErrorKind::RecordNotFound => response_not_found(&e.message),
        Err(e) => response_error(&e.message),
    }
}

pub async fn single(
    State(state): State<AppState>,
    Extension(session): Session,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state
        .holding_service
        .single_for_user(&session.user_id, id)
        .await
    {
        Ok(record) => response_ok(record),
        Err(e) => response_error(e.as_str()),
    }
}

/*
pub async fn list(
    State(state): State<AppState>,
    Extension(session): Session) -> impl IntoResponse {
    match state.holding_service.list_for_user(&session.user_id).await {
        Ok(entities) => response_ok(entities),
        /*{
            let models = entities.into_iter()
                .map(|entity| entity.into())
                .collect::<Vec<models::Custodian>>();
        },*/
        Err(e) => response_error(e.as_str()),
    }
}*/

// TODO: move in helper file
macro_rules! get_user {
    ($state:expr, $session:expr) => {
        match $state.user_service.get(&$session.user_id).await {
            Ok(Some(user)) => user,
            Ok(None) => return response_bad_request("User not found"),
            Err(e) => return response_error(&e),
        }
    };
}

pub async fn list(
    State(state): State<AppState>,
    Extension(session): Session,
    Query(params): Query<models::search::Params>,
) -> impl IntoResponse {
    let user = get_user!(state, session);

    let result = match params.only_latest_balance {
        true => state.holding_service.list_last_balance(&user).await,
        _ => state.holding_service.list_for_user(&user).await,
    };

    match result {
        Ok(entities) => response_ok(entities),
        /*{
            let models = entities.into_iter()
                .map(|entity| entity.into())
                .collect::<Vec<models::Custodian>>();
        },*/
        Err(e) => response_error(e.as_str()),
    }
}
