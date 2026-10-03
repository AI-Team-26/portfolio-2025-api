# TODO

- Feature 5 [refactor/05_thiserror_errors] Replace manual error types with `thiserror` derives: convert `DatabaseError`/`ErrorKind` (repositories/errors.rs) and service errors (`AuthError`, `LoginError`,
 `CreateError`) to enums using `#[derive(thiserror::Error)]` with `#[from]` conversions; remove dead manual constructors and `.map_err()` boilerplate at call sites

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

- Feature 15 [perf/15_single_refresh_lookup] Collapse double DB query in AuthService.refresh_session(): remove pre-check via SessionRepository.exists_by_refresh_token and rely solely on find_by_refresh_token (None → InvalidOrExpiredToken); delete exists_by_refresh_token method if unused elsewhere. In the same pass, scrub the raw refresh token from error/log strings ("NOT FOUND - 111 | refresh_token: {}" currently embeds the live token — security leak flagged in Analysis.md); use fixed messages without secrets. Verify no behavior change for valid/expired tokens; covered by existing integration tests + new unit test once Epic 13 trait seams land.

- Feature 16 [perf/16_batch_insert_currency_rates] Fix N+1 in src/jobs/update_currency_rates_job.rs: job currently loops `for rate in rates { currency_rate_service.create(&rate) }` issuing one INSERT per coin per run. Add bulk upsert path: CurrencyRateService::create_many → CurrencyRateRepository batch insert using single `INSERT ... ON CONFLICT DO NOTHING` (or sqlx query builder batching); aggregate failures instead of per-item logging (log count + symbols failed); verify idempotency since job runs periodically; measure before/after job duration and round-trips.

- Feature 17 [refactor/17_token_lifetimes_config] Make hardcoded token lifetimes configurable: move ACCESS_TOKEN_LIFETIME (30 min) and REFRESH_TOKEN_LIFETIME (30 days) out of src/constants.rs into Configuration fields (e.g. auth_access_token_lifetime_secs default 1800, auth_refresh_token_lifetime_secs default 2592000); constants become fallback defaults only; wire through AuthService/session creation where they're consumed. Composes with refactor/07_config_crate — implement there if that lands first, otherwise extend current JSON loading. Document trade-off: longer refresh = fewer logins vs larger revocation window.

- Feature 18 [feat/18_request_timeout_middleware] Add HTTP request timeout protection: wrap router with tower `TimeoutLayer` (tower::timeout) returning 504 Gateway Timeout when a handler exceeds budget (start 30s global; allow per-route override later if needed). Note complementarity with db acquire_timeout (refactor/08): pool wait is bounded separately so slow handlers fail fast rather than hang connections. Ensure background jobs are unaffected (they don't go through the HTTP layer).

- Feature 19 [feat/19_graceful_shutdown] Implement graceful shutdown in main.rs: axum `serve(...).with_graceful_shutdown(signal)` on SIGINT/SIGTERM via tokio::signal; sequence: stop accepting new connections → drain in-flight requests with bounded wait (~10-30s then force exit) → cancel scheduled cron jobs via job_manager handle → close PgPool cleanly (pool.close()) → log each phase. Critical for Docker deploys where orchestrator sends SIGTERM to old container during rollout.

- Feature 20 [chore/20_modernize_deps] Drop two obsolete dependencies using std/stable replacements: (a) once_cell → std::sync::LazyLock in src/services/Coingecko/currencies_map.rs (Lazy→LazyLock drop-in rename), remove once_cell from Cargo.toml; (b) async-trait → native async fn in traits (stable since Rust 1.75): remove #[async_trait] attributes and imports in src/jobs/job_manager.rs and src/jobs/update_currency_rates_job.rs, update the outdated "not yet natively supported" comment; watch for dyn-dispatch seams that may need explicit future boxing. Behavior-preserving; cargo build + clippy + tests green before merge.

- Feature 21 [feat/21_security_headers_ratelimit] Harden API security surface: (a) in-app rate limiting with tower-governor GovernorLayer on /auth/* routes (~10 req/min per client, keyed from Cf-Connecting-Ip/XFF first hop per fix/09); (b) security response headers via tower-http SetResponseHeader layer: Content-Security-Policy (tuned to frontend origin), X-Content-Type-Options: nosniff, Referrer-Policy: no-referrer; (c) document that HSTS + edge brute-force rules belong in Cloudflare config (Always Use HTTPS + WAF rate-limit rule on /auth/*) — provide exact CF dashboard steps in devop/README.md rather than code. Verify headers present in curl -I responses end-to-end through nginx.

## Done

- Feature 5.1 | Analyze error management. Identify bugs, duplication and bad code. See FEATURE_5_1_ERROR_MANAGEMENT_ANALYSIS.md.
- Feature 4