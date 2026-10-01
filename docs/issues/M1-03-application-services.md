# M1-03: Application Services

**Status:** complete  
**Dependencies:** M1-01, M1-02

## Objective

Expose create, execute, retrieve, and list use-cases behind framework-independent persistence ports.

## Scope

Define repository interfaces and in-memory implementations for tests; persist lifecycle events at each transition. Exclude SQL, Axum, Tauri, and frontend code.

## Implementation plan

1. Define repository methods and application error categories.
2. Implement create, execute, get, and list services.
3. Persist created, started, and terminal events.
4. Test with an in-memory adapter.

## Verification

- Execute happy and rejection use-cases solely through application services.
- Confirm ordering and content of the saved event timeline.

## Acceptance criteria

- Application code depends only on domain plus port abstractions.
- Every terminal state is persisted.
- Tests run with no processes or containers.

## Evidence

- `RunService` implements create, execute, retrieve, and list use-cases through `RunRepository`; ID creation is isolated behind `RunIdGenerator`.
- `InMemoryRunRepository` stores complete snapshots under a mutex and returns stable ID ordering. Created, started, and terminal snapshots are saved at each transition.
- `cargo test -p local-stack-proof-application` passed: 5 tests covering completed and rejected flows, saved event order/content, retrieve/list, repeated execution rejection, and missing IDs.
- `cargo clippy -p local-stack-proof-application --all-targets -- -D warnings` passed.
- Application crate dependencies remain limited to the domain crate; no processes or containers are needed.
