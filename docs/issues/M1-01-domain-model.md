# M1-01: Domain Model

**Status:** complete  
**Dependencies:** M0-01

## Objective

Define typed, framework-free demo-run state and its invariants.

## Scope

Implement IDs, configuration, status, events, result, validation messages, and legal transitions in the domain crate. Exclude persistence, HTTP, Tauri, and React.

## Implementation plan

1. Model run lifecycle: queued, running, completed, rejected, failed.
2. Make configuration snapshots immutable after creation.
3. Encode result/state invariants in constructors or transitions.
4. Add clear domain errors and serialization only where future boundaries require it.

## Verification

- Run domain unit tests for all legal and illegal transitions.
- Test that incomplete runs cannot hold results.

## Acceptance criteria

- Statuses are types/enums, not unconstrained strings.
- Invalid transitions return a domain error or are impossible to represent.
- The crate has no transport, host, or database dependency.

## Evidence

- `cargo test -p local-stack-proof-domain`: 4 lifecycle and invariant tests passed.
- `cargo clippy -p local-stack-proof-domain --all-targets -- -D warnings`: passed.
- `cargo fmt --all`: applied; formatting check passed in the final validation.
