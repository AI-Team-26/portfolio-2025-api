# Feature 5.1 — Error-management analysis

## Scope and conclusion

This review examines the error handling on `main` and compares the two open Feature 5 pull requests. It does **not** implement the refactor.

The current code has concrete correctness and maintainability problems. `thiserror` is justified for a focused part of the work, but it is not a reason to convert every `String` return type or touch 25–28 files at once.

Recommended conclusion:

- **Do use `thiserror`** for errors crossing repository, service, and HTTP boundaries where callers need to distinguish cases such as duplicate data, not-found data, invalid credentials, and infrastructure failure.
- **Do not use `thiserror` as a mechanical replacement** for every `String`. A descriptive string is acceptable for a local conversion/helper failure until the boundary and recovery behavior are defined.
- **Separate correctness fixes from the type migration.** The refresh-token query bug, swallowed database errors, and token leakage need focused security/correctness work and tests.

## Real issues found on `main`

### 1. Refresh-token lookup is incorrect and hides failures — high severity

`src/repositories/session_repository.rs::find_by_refresh_token` ignores its argument and queries `WHERE id = 72`. It therefore does not look up the supplied refresh token.

The same method formats SQL errors into `String`, while `AuthService::refresh_session` uses `.ok().flatten()`. A database outage can consequently be treated as if the token were absent instead of being returned as an infrastructure error.

`AuthService::refresh_session` also performs an existence query and then a lookup/update sequence. This is duplicate work and creates multiple race windows.

**Recommendation:** one focused security/correctness change:

1. bind the refresh token in the lookup;
2. propagate lookup errors;
3. collapse the pre-check and lookup where possible;
4. add tests for valid, absent/expired, and database-failure paths.

This should not be hidden inside a broad error-type migration.

### 2. A live refresh token is included in error text — high severity

`AuthService::refresh_session` includes `refresh_token` in error strings. Tokens must not appear in API responses, logs, or diagnostic error values.

**Recommendation:** replace the value with a fixed invalid/expired-token message and keep detailed database diagnostics internal.

### 3. Error contracts are inconsistent — medium/high severity

Repositories return a mixture of `Result<_, String>` and `Result<_, DatabaseError>`. Services similarly mix `String`, `DatabaseError`, and service-specific enums. Endpoint code therefore relies on string formatting in some places and `ErrorKind` checks in others.

Examples include:

- `CurrencyRepository`, `UserRepository`, `SessionRepository`, and several others returning `String`;
- `CustodianRepository` and `HoldingRepository` using `DatabaseError` for only some methods;
- `CustodianService::create` flattening a structured database error to `Unexpected(String)`;
- endpoints mapping errors based on `kind`, while other endpoint paths expose stringified errors.

This makes it easy to accidentally turn a not-found or duplicate condition into a 500 response and makes error handling hard to test.

**Recommendation:** define the error contract at each boundary first, then migrate one vertical slice at a time. Preserve context with `#[source]`/transparent variants or explicit context messages rather than flattening everything.

### 4. Duplicate-user checking is race-prone — medium severity

`UserService::create` checks whether a username exists and then inserts it. Concurrent requests can both pass the check; the database unique constraint must remain the authority. The pre-check also adds a query and can produce a different error than the insert.

**Recommendation:** rely on the unique constraint for correctness, translate its violation into a typed `UsernameAlreadyInUse`, and retain the pre-check only if its UX benefit is demonstrated and its race limitation is documented. Add a database-error mapping test.

### 5. Error messages contain avoidable noise and inconsistent context — low/medium severity

There are typos (`Currncies`, `Cistodian`, `Sessoon`), inconsistent capitalization, repeated `Failed to ...` wrappers, and commented-out implementations around active repository code. These are readability problems, but changing them should not be mixed with behavior changes unless the affected call site is being migrated.

### 6. Some failures still panic or are silently discarded — medium severity, broader scope

Examples include `unwrap`/`expect` in request parsing, scheduler startup, configuration loading, header creation, and token/password paths. Some are valid startup invariants; others are request-dependent and can crash or hide a useful error.

**Recommendation:** audit these separately. Classify each as an invariant, startup failure, or request/data failure before replacing it. Do not blindly replace all `unwrap` calls with a generic error.

## Does `thiserror` make this code smaller and clearer?

Yes, but only after the error boundaries are designed.

The current manual `DatabaseError` stores a message plus a separate `ErrorKind`, with constructors that duplicate the representation. A `thiserror` enum can encode the same state directly:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("record not found")]
    NotFound,
    #[error("duplicate field: {0}")]
    Duplicate(String),
    #[error("database operation failed")]
    Sql(#[source] sqlx::Error),
}
```

This removes the parallel `kind` field and constructor boilerplate, supplies `Display`/`Error`, and allows `?` at service boundaries. It also makes exhaustive matching possible and avoids fragile comparisons against formatted strings.

However, `thiserror` does **not** automatically improve the design:

- wrapping every SQL error as `Generic(String)` loses the original source and prevents reliable classification;
- `#[from]` is useful only when the conversion preserves the intended semantics;
- nested service/database variants can produce noisy messages;
- converting all repository methods at once increases review surface without proving behavior preservation.

A smaller, readable change is therefore possible, but it should be a staged vertical refactor rather than a crate-wide mechanical rewrite.

## Comparison of PR 17 and PR 20

Both PRs convert the same broad area and add `thiserror`; neither is a small Feature 5.1 analysis-sized change. Their changes overlap heavily across repositories, services, endpoints, dependency metadata, and SQLx offline data.

### PR 17 (`refactor/05_thiserror_errors`)

**Worth retaining:**

- typed service errors and `?` propagation reduce real `.map_err()` boilerplate;
- endpoint handling can distinguish not-found from generic failures without parsing strings;
- it identifies and fixes the hard-coded `id = 72` refresh lookup and stops swallowing its error;
- it was reviewed through several iterations and restores compile-time `query_as!` checking.

**Risks/concerns:**

- the refresh lookup bug fix and token/error behavior changes are mixed into the broad refactor;
- the PR touches about 25 files, making it difficult to establish that unrelated behavior is unchanged;
- SQL errors are generally converted to `DatabaseError::Generic(String)`, so source errors and consistent duplicate classification are still lost;
- some service error variants and conversions are dead or exist only to preserve an awkward pre-existing contract;
- an unrelated empty `BaseRepository` implementation remains under unresolved review discussion.

PR 17 is the more complete and reviewed implementation, but its correctness fixes should be extracted or explicitly split before adopting the type migration.

### PR 20 (`refactor/05_thiserror_errors_agent`)

**Worth retaining:**

- the same basic enum direction and endpoint typed matching;
- simpler `?` propagation in several service methods;
- it removes the dead `From<sqlx::Error>` implementation rather than implying it classifies errors that callers immediately stringify.

**Risks/concerns:**

- it is similarly broad (about 28 files including metadata/backlog changes) and has less review evidence;
- it keeps the duplicate refresh existence query and does not by itself address the security leak in refresh-token error text;
- most SQL errors are still eagerly flattened into `DatabaseError::Generic(String)`;
- changing `TODO.md` and including unrelated repository-trait changes reduces scope clarity;
- the same hard-coded refresh lookup is fixed as part of the broad change rather than isolated and tested.

PR 20 is not a preferable minimal alternative; it mainly represents a shorter implementation of the same broad migration.

## Proposed split

The work should be split into precise, independently reviewable changes:

1. **Fix refresh-session correctness and secret handling** (2–4 files): bind the token, stop swallowing database errors, remove the double lookup if safe, and remove token values from errors/logs. Add focused tests.
2. **Define and migrate repository error contracts** (a small vertical slice, for example user creation): preserve SQL sources, map unique violations, and test duplicate/not-found/database-failure behavior.
3. **Migrate authentication/session service errors** (auth, session, and their endpoints): use `thiserror` and `#[from]` only where the conversion is semantically lossless. Add service-level tests before expanding.
4. **Migrate remaining CRUD slices** (currency, custodian, holding, and currency-rate), at most 2–3 repositories per change. Remove dead constructors/comments only where touched.
5. **Separate panic/unwrap audit** into its own reliability task, classifying startup invariants versus request failures.

This sequence delivers real bug fixes first, demonstrates whether `thiserror` reduces code in a representative slice, and keeps each review below the size of PR 17/20.

## Decision

Do not merge either PR wholesale as the implementation of Feature 5.1. Use their useful findings—especially the refresh-token query defect, error propagation, and typed endpoint matching—as input to the staged changes above. The first implementation candidate should be the focused refresh-session correctness/security fix, followed by a small typed-error vertical slice.
