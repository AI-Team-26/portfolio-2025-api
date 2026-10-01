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