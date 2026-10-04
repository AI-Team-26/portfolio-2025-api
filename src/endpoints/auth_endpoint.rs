use crate::{debug, info};

use crate::{
    endpoints::{models::auth_models::refresh_token, request_json_validator::ValidJson},
    services::auth_service::{AuthError, LoginError, LoginRequest},
};
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    response::IntoResponse,
};

use crate::services::user_service::CreateError;
use crate::{
    endpoints::{
        models::auth_models::{login, signup},
        response_utils::*,
    },
    state::AppState,
};

/// Extracts client IP and User-Agent from HTTP headers.
/// Prefers Cf-Connecting-Ip (set authoritatively by Cloudflare), falls back to first hop of X-Forwarded-For.
#[derive(Debug)]
pub(crate) struct ClientMetadata {
    pub ip_address: String,
    pub user_agent: String,
}

impl<S> FromRequestParts<S> for ClientMetadata
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Prefer Cf-Connecting-Ip (Cloudflare authoritative), fall back to first hop of X-Forwarded-For
        let cf_ip: Option<&str> = parts
            .headers
            .get("cf-connecting-ip")
            .and_then(|v| v.to_str().ok());

        let ip_address = match cf_ip {
            Some(ip) if !ip.is_empty() => ip.to_string(),
            _ => {
                let xff: Option<&str> = parts
                    .headers
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok());
                let first_hop: Option<String> = xff
                    .and_then(|v| v.split(',').next())
                    .map(|h| h.trim().to_string())
                    .filter(|s| !s.is_empty());
                first_hop.unwrap_or_else(|| "unknown".into())
            }
        };

        let ua: Option<&str> = parts
            .headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok());
        let user_agent = ua
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            ip_address,
            user_agent,
        })
    }
}

pub async fn signup(
    State(state): State<AppState>,
    ValidJson(request): ValidJson<signup::Request>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&request) {
        return *response;
    }

    let Some(currency) = state.currency_service.try_get(request.currency_id) else {
        return response_bad_request(&format!(
            "Currency not found with ID={}",
            request.currency_id
        ));
    };

    match state
        .auth_service
        .signup(request.username, request.password, currency)
        .await
    {
        Ok(_) => response_ok(signup::Response::success()),
        Err(CreateError::UsernameAlreadyInUse) => {
            response_ok(signup::Response::error("Username already taken"))
        }
        Err(CreateError::DatabaseError(e)) => {
            // TODO: log error
            response_error(&e)
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    client_meta: ClientMetadata,
    ValidJson(request): ValidJson<login::Request>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&request) {
        return *response;
    }

    info!("login");

    let username = request.username.trim().to_string();
    let password = request.password.trim().to_string();

    debug!(
        "login from ip={} ua={}",
        client_meta.ip_address, client_meta.user_agent
    );

    let service_request = LoginRequest {
        username,
        password,
        ip_address: client_meta.ip_address,
        user_agent: client_meta.user_agent,
    };

    match state.auth_service.login(service_request).await {
        Ok(session) => response_ok(login::Response::from(session)),
        Err(LoginError::FailedLogin) => response_unhautorized("Wrong username or password"),
        Err(LoginError::DatabaseError(e)) => response_error(&e),
    }
}

pub async fn refresh_token(
    State(state): State<AppState>,
    ValidJson(request): ValidJson<refresh_token::Request>,
) -> impl IntoResponse {
    if let Err(response) = validate_request(&request) {
        return *response;
    }

    info!("refresh_token");
    match state
        .auth_service
        .refresh_session(request.refresh_token)
        .await
    {
        Ok(session) => response_ok(refresh_token::Response::from(session)),
        Err(AuthError::InvalidOrExpiredToken(data)) => response_invalid_token(
            format!("Refresh token is invalid or expired. {}", data).as_str(),
        ),
        Err(AuthError::DatabaseError(e)) => response_error(&e),
    }
}
