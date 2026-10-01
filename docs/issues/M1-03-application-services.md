# M1-03: Application Services

**Status:** planned
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


