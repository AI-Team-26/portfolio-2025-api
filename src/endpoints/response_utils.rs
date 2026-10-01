use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::constants;
use crate::endpoints::models::common::{ErrorResponse, NewIdResponse, ValidationErrorsResponse};

pub fn response_ok_no_data() -> Response {
    (StatusCode::OK).into_response()
}

pub fn response_ok<T: serde::Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(data)).into_response()
}

pub fn response_ok_map<T, U>(data: impl IntoIterator<Item = T>, _target: fn(T) -> U) -> Response
where
    U: Serialize + From<T>,
{
    let mapped: Vec<U> = data.into_iter().map(U::from).collect();
    response_ok(mapped)
}

#[allow(dead_code)]
pub fn response_created<T: serde::Serialize>(data: T) -> Response {
    (StatusCode::CREATED, Json(data)).into_response()
}

pub fn response_created_new_id(new_id: i32) -> Response {
    (StatusCode::CREATED, Json(NewIdResponse { new_id })).into_response()
}

// Errors
pub fn response_error(message: &str) -> Response {
    response_error_code(StatusCode::INTERNAL_SERVER_ERROR, message, None)
}

pub fn response_bad_request(message: &str) -> Response {
    response_error_code(StatusCode::BAD_REQUEST, message, None)
}

pub fn response_validation_errors(errors: Vec<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ValidationErrorsResponse { errors }),
    )
        .into_response()
}

pub fn response_duplicated_value(field: &str) -> Response {
    response_error_code(
        StatusCode::CONFLICT,
        &format!("{}: already exists", field),
        None,
    )
}

pub fn response_not_found(message: &str) -> Response {
    response_error_code(StatusCode::NOT_FOUND, message, None)
}

pub fn response_invalid_token(message: &str) -> Response {
    response_error_code(
        StatusCode::UNAUTHORIZED,
        message,
        Some(constants::auth::error_codes::INVALID_OR_EXPIRED_TOKEN),
    )
}

pub fn response_missing_auth_header(message: &str) -> Response {
    response_error_code(
        StatusCode::UNAUTHORIZED,
        message,
        Some(constants::auth::error_codes::MISSING_AUTH_HTTP_HEADER),
    )
}

pub fn response_unhautorized(message: &str) -> Response {
    response_error_code(StatusCode::UNAUTHORIZED, message, None)
}

fn response_error_code(status_code: StatusCode, message: &str, code: Option<&str>) -> Response {
    match code {
        Some(code) => (status_code, Json(ErrorResponse::error_code(message, code))).into_response(),
        _ => (status_code, Json(ErrorResponse::error(message))).into_response(),
    }
}
