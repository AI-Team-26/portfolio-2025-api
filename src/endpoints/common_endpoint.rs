use axum::response::IntoResponse;

use crate::{
    endpoints::{models::common::ConfigResponse, response_utils::response_ok},
    entities::custodian::KINDS,
};

// Axum handles the conversion of a simple string to the HTTP response
pub async fn home() -> &'static str {
    "Hello, Axum API (learning.Rust)!"
}

pub async fn config(/*State(state): State<AppState>*/) -> impl IntoResponse {
    let config_response = ConfigResponse {
        custodian_kinds: KINDS.iter().map(|s| s.to_string()).collect(),
    };

    response_ok(config_response)
}
