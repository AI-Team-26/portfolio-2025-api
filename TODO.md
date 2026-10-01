# TODO

- Feature 4 [refactor/04_extract_appstate] Extract `AppState` from `utils/dependency_injection.rs` into a dedicated `src/state.rs` module; update endpoint imports (`crate::dependency_injection::AppState` →
 `crate::state::AppState`) so DI wiring no longer owns the state type

- Feature 5 [refactor/05_thiserror_errors] Replace manual error types with `thiserror` derives: convert `DatabaseError`/`ErrorKind` (repositories/errors.rs) and service errors (`AuthError`, `LoginError`,
 `CreateError`) to enums using `#[derive(thiserror::Error)]` with `#[from]` conversions; remove dead manual constructors and `.map_err()` boilerplate at call sites

- Feature 6 [refactor/06_validator_crate] Replace custom `validate!` macro and `RuleString`/`RuleNumber`/`RuleDate` enums (src/endpoints/request_validator.rs) with the `validator` crate: annotate request
 models in endpoints/models/* with derive-based validation, swap macro call sites in endpoints for `.validate()` + shared 4xx mapping, keep custom domain rules (custodian kind, not-in-future date) as custom
 validators; delete request_validator.rs

- Feature 7 [refactor/07_config_crate] Replace hand-rolled JSON loading (Configuration::load_from_json_file) with the `config` crate: layered precedence env > file > defaults via Config::builder +
 Environment::with_prefix("APP").separator("__"); drop CONFIGURATION_FILE path requirement, keep configuration_example.json as template, allow #[serde(default)] fallbacks on non-critical fields

- Feature 8 [refactor/08_db_pool_tuning] Replace PgPool::connect defaults in main.rs with explicit PgPoolOptions: add db_max_connections (start 20), db_min_connections (start 2-5) and acquire_timeout (e.g.
 5s) to Configuration; document rationale (backpressure vs Postgres process-per-connection cost) and tune based on observed pool saturation under real load

- Feature 9 [fix/09_session_client_metadata] Populate real client metadata at login (auth_endpoint.rs currently stores empty strings): read IP from X-Forwarded-For first hop (trusted-proxy chain:
 Cloudflare → nginx; consider Cf-Connecting-Ip as primary since Cloudflare sets it authoritatively) and User-Agent from headers into LoginRequest; validate non-empty before persisting to Sessions

- Feature 10 [feat/10_health_endpoint] Add GET /health endpoint returning 200 "OK". Two-tier design: (a) liveness probe — always 200 when process is up, used by orchestrators to decide restarts; (b)
 readiness check via ?ready=true query param (or separate /readyz route) that runs SELECT 1 against PgPool with short timeout (~2s) and returns 503 if DB unreachable. Register route before auth middleware so
 it's unauthenticated. Update Dockerfile HEALTHCHECK to curl /health. Document expected behavior behind Cloudflare/nginx (proxy should bypass cache for this route).

- Feature 11 [feat/11_metrics_endpoint] Add GET /metrics exposing Prometheus-format metrics using axum-prometheus (add axum-prometheus dep): per-route request count histogram
 (http_requests_total{method,route,status}, http_request_duration_seconds), in-flight requests gauge, and sqlx pool gauges (size, idle, waiting_tasks) polled from PgPoolMetrics — enables measuring pool
 saturation to inform refactor/08_db_pool_tuning values. Route must be excluded from auth middleware; restrict access at nginx level (allow internal network/scrapers only, deny public) since metric labels can
 leak info. Add scrape interval guidance (15-30s) and note that /metrics responses must not be cached by Cloudflare.

- Feature 12 [feat/12_opentelemetry_tracing] Integrate distributed tracing: add opentelemetry, opentelemetry_sdk, opentelemetry-otlp, tracing-opentelemetry deps. Create TracerProvider with OTLP exporter
 (endpoint configurable via config layer, e.g. OTEL_EXPORTER_OTLP_ENDPOINT env var); wrap tokio runtime with tracer subscriber so existing tracing spans propagate W3C traceparent headers on outbound HTTP
 calls (Coingecko API client) and inject TraceId into the existing JSON logging format (tracing-subscriber json writer customizer or field injection) so every log line carries the trace ID even without a
 collector deployed. Phase 1 (this PR): local span hierarchy (HTTP handler → service → repository SQL) + trace ID in logs. Phase 2 (follow-up, needs infra): OTLP export to Grafana Tempo/Jaeger — blocked until
 an OTLP collector exists in devops stack; leave exporter disabled-by-default via configuration flag enable_distributed_tracing (default false).

- Epic 13 | Introduce mocking and service-level unit tests (mockall + trait seams; currently only pure-function units exist, services/repositories untested)
  - Feature 13.1 [refactor/13_1_user_repo_trait] Introduce trait seam for ONE repository: define `trait UserRepository` (find_by_username, create) in src/repositories/user_repository.rs with
 #[cfg_attr(test, mockall::automock)]; rename struct → UserRepositoryImpl implementing the trait; UserService/AuthService constructors take Arc<dyn UserRepository>; wire impl at startup in main.rs/DI. Add
 mockall = "0.13". Behavior-preserving; decide async-fn-in-trait approach (async-trait vs boxed futures). Validates pattern before rollout.
  - Feature 13.2 [test/13_2_auth_login_tests] Unit tests for AuthService.login() using MockUserRepository: happy path, wrong password → FailedLogin, unknown user → FailedLogin, database error propagation.
 Offline (no Postgres), plain cargo test. Prerequisite: 13.1 merged.
  - Feature 13.3 [refactor/13_3_remaining_traits] Roll out validated pattern to remaining repositories (session, currency, custodian, holding, currency_of_user): extract traits, *Impl renames, Arc<dyn
 Trait> injection, DI updates. Max 2-3 repos per PR. No new tests here.
  - Feature 13.4 [test/13_4_service_coverage] Extend suites to remaining flows: AuthService (signup duplicate, refresh rotation/expiry), SessionService (create/expire/purge), Currency/Custodian CRUD incl.
 DatabaseError variants. Every public service method gets happy-path + primary failure-mode coverage via cargo test, zero external deps. Composes with refactor/05_thiserror_errors (do that first if possible
 for clean typed assertions).