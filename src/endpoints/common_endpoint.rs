use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::time::Duration;

use crate::{
    endpoints::{models::common::ConfigResponse, response_utils::response_ok},
    entities::custodian::KINDS,
    state::AppState,
};

// Axum handles the conversion of a simple string to the HTTP response
pub async fn home() -> &'static str {
    "Hello, Axum API (learning.Rust)!"
}

#[derive(Deserialize)]
pub struct HealthParams {
    ready: Option<bool>,
}

/// Liveness probe: 200 as long as the process is up.
/// With `?ready=true` also verifies DB connectivity (`SELECT 1`, ~2s timeout)
/// and returns 503 when the database is unreachable.
pub async fn health(
    State(state): State<AppState>,
    Query(params): Query<HealthParams>,
) -> impl IntoResponse {
    if !params.ready.unwrap_or(false) {
        return StatusCode::OK.into_response();
    }

    let check = tokio::time::timeout(
        Duration::from_secs(2),
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db_pool),
    )
    .await;

    match check {
        Ok(Ok(_)) => StatusCode::OK.into_response(),
        _ => {
            tracing::warn!("Readiness check failed: database unreachable or timed out");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

pub async fn config(/*State(state): State<AppState>*/) -> impl IntoResponse {
    let config_response = ConfigResponse {
        custodian_kinds: KINDS.iter().map(|s| s.to_string()).collect(),
    };

    response_ok(config_response)
}
