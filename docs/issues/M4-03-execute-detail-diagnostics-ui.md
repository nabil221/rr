# M4-03: Execute, Detail, and Diagnostics UI

**Status:** planned
**Dependencies:** M4-02

## Objective

Make run execution observable and make active host/storage mode inspectable.

## Scope

Build execute action, polling status refresh, configuration snapshot, event timeline, results/validation states, and safe diagnostics. Exclude WebSockets and PWA caching.

## Verification

- Complete a run without manual refresh.
- Run controlled rejection and confirm it differs from a zero-value result.
- Confirm Diagnostics identifies web-local/Postgres.

## Acceptance criteria

- Detail view can recreate a prior persisted run after refresh.
- Polling stops in terminal state.
- No secrets, paths, or stack traces appear in Diagnostics.


