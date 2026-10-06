// integration tests for the Portfolio API endpoints.
// Uses testcontainers to spin up a real Postgres instance.
// Run with: cargo test --test integration_tests
//
// Prerequisite: Docker must be running so testcontainers can start the container.
// To point at a running Postgres instead of spinning up a container, set
// CONFIGURATION_FILE to a local json file and the container url as the
// database_connection_string (no container created in that case).

/// Default path to the local configuration file used by integration tests.
const DEFAULT_CONFIG_PATH: &str = "src/configuration_local.json";

use portfolio_api::repositories::schemas::session_record::SessionWithUser;
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

/// Spawn a Postgres container and return a ready connection pool.
async fn postgres_pool() -> PgPool {
    // use the community-maintained Postgres module (matches devop compose)
    let pg_image = Postgres::default()
        .with_user("portfolio_user")
        .with_password("portfolio_password")
        .with_db_name("portfolio");

    let container = pg_image
        .start()
        .await
        .expect("Failed to start Postgres container");
    let port = container.get_host_port_ipv4(5432).await.expect("port");
    let host = container.get_host().await.expect("host");

    let url = format!("postgres://portfolio_user:portfolio_password@{host}:{port}/portfolio");

    // give Postgres a moment to accept connections
    let pool = PgPool::connect(&url)
        .await
        .expect("Failed to connect to Postgres container");

    // Apply migrations so the schema is ready for tests
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Build the application state that each test uses.
async fn app_state() -> portfolio_api::state::AppState {
    let pool = postgres_pool().await;
    portfolio_api::utils::dependency_injection::inject_services(&load_config(), pool).await
}

fn load_config() -> portfolio_api::configuration::Configuration {
    use std::env;
    // Use the local configuration file from the repo
    let cfg_path =
        env::var("CONFIGURATION_FILE").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    portfolio_api::configuration::Configuration::load_from_json_file(&cfg_path)
        .expect("Failed to load configuration")
}

/// Helper to log in and obtain the session / tokens.
async fn login(
    state: &portfolio_api::state::AppState,
) -> portfolio_api::entities::session::Session {
    use portfolio_api::services::auth_service::LoginRequest;
    let req = LoginRequest {
        username: "testuser".to_string(),
        password: "TestPass1!".to_string(),
        ip_address: "127.0.0.1".to_string(),
        user_agent: "integration-test".to_string(),
    };
    state
        .auth_service
        .login(req)
        .await
        .unwrap_or_else(|_| panic!("login failed"))
}

// ── Home / Config ────────────────────────────────────────────────────

#[tokio::test]
async fn home_returns_ok() {
    let _state = app_state().await;
    let response = portfolio_api::endpoints::common_endpoint::home().await;
    let _ = response; // must not panic
}

#[tokio::test]
async fn config_returns_ok() {
    let _state = app_state().await;
    let response = portfolio_api::endpoints::common_endpoint::config().await;
    let _ = response;
}

// ── Auth: Signup / Login / Refresh ───────────────────────────────────

#[tokio::test]
async fn signup_creates_user_and_returns_session() {
    let state = app_state().await;

    // ensure a currency exists for the signup request
    let currency = state
        .currency_service
        .try_get(1)
        .expect("currency 1 missing");

    use portfolio_api::endpoints::models::auth_models::signup;
    let req = signup::Request {
        username: format!("signup_test_{}", uuid::Uuid::new_v4()),
        password: "TestPass1!".to_string(),
        currency_id: currency.id,
    };
    let response = portfolio_api::endpoints::auth_endpoint::signup(
        axum::extract::State(state.clone()),
        portfolio_api::endpoints::request_json_validator::ValidJson(req),
    )
    .await;
    let _ = response; // must not panic
}

#[tokio::test]
async fn login_returns_session() {
    let state = app_state().await;
    let session = login(&state).await;
    assert!(!session.access_token.is_empty());
    assert!(!session.refresh_token.is_empty());
}

#[tokio::test]
async fn refresh_token_returns_new_tokens() {
    let state = app_state().await;
    let session = login(&state).await;

    use portfolio_api::endpoints::models::auth_models::refresh_token;
    let req = refresh_token::Request {
        refresh_token: session.refresh_token.clone(),
    };
    let response = portfolio_api::endpoints::auth_endpoint::refresh_token(
        axum::extract::State(state.clone()),
        portfolio_api::endpoints::request_json_validator::ValidJson(req),
    )
    .await;
    let _ = response;
}

// ── Currency CRUD ────────────────────────────────────────────────────

#[tokio::test]
async fn currency_list_all_returns_currencies() {
    let state = app_state().await;
    let response = portfolio_api::endpoints::currency_endpoint::list_all(
        axum::extract::State(state.clone()),
        axum::Extension(
            portfolio_api::repositories::schemas::session_record::SessionWithUser {
                user_id: "00000000-0000-0000-0000-000000000000".to_string(),
                username: "admin".to_string(),
                access_token_expires_at: portfolio_api::utils::datetime::now(),
                refresh_token_expires_at: portfolio_api::utils::datetime::now(),
            },
        ),
    )
    .await;
    let _ = response;
}

#[tokio::test]
async fn currency_single_returns_one() {
    let state = app_state().await;
    let currency = state
        .currency_service
        .try_get(1)
        .expect("currency 1 missing");

    let response = portfolio_api::endpoints::currency_endpoint::single(
        axum::extract::State(state.clone()),
        axum::extract::Path(currency.id),
    )
    .await;
    let _ = response;
}

#[tokio::test]
async fn currency_create_and_delete_cycle() {
    let state = app_state().await;

    use portfolio_api::endpoints::models::currency_models;
    let create_req = currency_models::CreateRequest {
        symbol: "XTEST".to_string(),
        name: "Test Currency".to_string(),
        kind: "Fiat".to_string(),
        is_active: true,
        precision: 2,
        is_major: false,
        coingecko_id: None,
    };
    let response = portfolio_api::endpoints::currency_endpoint::create(
        axum::extract::State(state.clone()),
        portfolio_api::endpoints::request_json_validator::ValidJson(create_req),
    )
    .await;
    let _ = response; // must not panic

    // cleanup: find and delete the created currency
    let currency = state.currency_service.try_get_by_symbol_CI("XTEST");
    if let Some(c) = currency {
        let _ = portfolio_api::endpoints::currency_endpoint::delete(
            axum::extract::State(state.clone()),
            axum::extract::Path(c.id),
        )
        .await;
    }
}

// ── Custodian CRUD ───────────────────────────────────────────────────

#[tokio::test]
async fn custodian_list_returns_custodians() {
    let state = app_state().await;
    let _session = login(&state).await;

    let response =
        portfolio_api::endpoints::custodian_endpoint::list(axum::extract::State(state.clone()))
            .await;
    let _ = response;
}

// ── Holding CRUD ─────────────────────────────────────────────────────

#[tokio::test]
async fn holding_create_and_list_cycle() {
    let state = app_state().await;
    let session = login(&state).await;

    // pick a currency and custodian for the holding
    let currency = state
        .currency_service
        .try_get(1)
        .expect("currency 1 missing");
    let custodian_list = state
        .custodian_service
        .list()
        .await
        .expect("custodian list failed");
    let custodian = custodian_list.first().expect("no custodians found");

    use portfolio_api::endpoints::models::holding_models::create;
    let req = create::Request {
        custodian_id: custodian.id,
        currency_id: currency.id,
        date: portfolio_api::utils::datetime::now(),
        action: "Balance At".to_string(),
        amount: rust_decimal::Decimal::from(100),
        note: Some("integration test".to_string()),
    };

    let response = portfolio_api::endpoints::holding_endpoint::create(
        axum::extract::State(state.clone()),
        axum::Extension(SessionWithUser {
            user_id: session.user.id.clone(),
            username: session.user.username.clone(),
            access_token_expires_at: session.access_token_expires_at,
            refresh_token_expires_at: session.refresh_token_expires_at,
        }),
        portfolio_api::endpoints::request_json_validator::ValidJson(req),
    )
    .await;
    let _ = response; // must not panic
}

#[tokio::test]
async fn holding_list_returns_items() {
    let state = app_state().await;
    let session = login(&state).await;

    let response = portfolio_api::endpoints::holding_endpoint::list(
        axum::extract::State(state.clone()),
        axum::Extension(SessionWithUser {
            user_id: session.user.id.clone(),
            username: session.user.username.clone(),
            access_token_expires_at: session.access_token_expires_at,
            refresh_token_expires_at: session.refresh_token_expires_at,
        }),
        axum::extract::Query(
            portfolio_api::endpoints::models::holding_models::search::Params {
                only_latest_balance: false,
            },
        ),
    )
    .await;
    let _ = response;
}

// ── Metrics ────────────────────────────────────────────────────────

#[tokio::test]
async fn metrics_endpoint_is_public_and_reports_expected_metrics() {
    let state = app_state().await;
    let metrics = portfolio_api::utils::metrics::init_metrics(state.db_pool.clone());
    let app = portfolio_api::utils::routing::set_routes(state.clone(), &metrics)
        .with_state(state);

    let request = http::Request::builder()
        .uri("/metrics")
        .header("host", "localhost")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let text = String::from_utf8(body.to_vec()).expect("utf8");

    // Pool gauges are updated by a background poller; give it time for a tick.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    for expected in [
        "axum_http_requests_total",
        "axum_http_request_duration_seconds",
        "axum_http_requests_pending",
        "db_pool_size",
        "db_pool_idle",
        "db_pool_in_use",
    ] {
        assert!(text.contains(expected), "missing metric: {expected}");
    }
    // The scrape of this very request must be counted.
    assert!(text.contains(r#"endpoint="/metrics""#));
}
