# TODO

- Feature 4 [refactor/04_extract_appstate] Extract `AppState` from `utils/dependency_injection.rs` into a dedicated `src/state.rs` module; update endpoint imports (`crate::dependency_injection::AppState` →
 `crate::state::AppState`) so DI wiring no longer owns the state type