# TODO

- Feature 4 [refactor/04_extract_appstate] Extract `AppState` from `utils/dependency_injection.rs` into a dedicated `src/state.rs` module; update endpoint imports (`crate::dependency_injection::AppState` →
 `crate::state::AppState`) so DI wiring no longer owns the state type

- Feature 5 [refactor/05_thiserror_errors] Replace manual error types with `thiserror` derives: convert `DatabaseError`/`ErrorKind` (repositories/errors.rs) and service errors (`AuthError`, `LoginError`,
 `CreateError`) to enums using `#[derive(thiserror::Error)]` with `#[from]` conversions; remove dead manual constructors and `.map_err()` boilerplate at call sites