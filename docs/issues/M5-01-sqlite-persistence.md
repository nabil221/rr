# M5-01: SQLite Persistence

**Status:** complete
**Dependencies:** M2-04

## Objective

Provide self-contained desktop persistence through the same repository port used by Postgres.

## Scope

Implement SQLite migrations, repository mapping, per-user application-data location, and test-directory override. Exclude web/database sync.

## Implementation plan

1. Add a synchronous bundled-SQLite adapter behind the existing repository and ID ports; keep SQL drivers out of the domain/application.
2. Reuse the verified storage snapshot mapping in a small shared persistence-format crate, not HTTP DTOs. Preserve Postgres format and tests.
3. Use versioned migrations, durable monotonic IDs, append-only events, and atomic snapshot/event writes. Serialize competing transitions with immediate transactions and a bounded busy timeout; reject stale state and corrupt readback safely.
4. Accept an explicit database path for isolated tests; resolve the ordinary per-user application-data location in the Tauri composition in M5-02.
5. Test reconnect, all lifecycle states, concurrent adapters, duplicate/stale saves, immutable events, corrupt data, and transaction rollback using unique temporary directories. No Docker/network prerequisite.

## Verification

- Run common repository-contract tests where applicable.
- Restart against the same test database and retrieve previous run history.

## Acceptance criteria

- Desktop state survives restart.
- Tests can create isolated data without touching ordinary user data.
- No desktop operation requires Postgres or network access.

## Evidence

- `cargo test -p local-stack-proof-persistence-sqlite -p local-stack-proof-persistence-format`: five isolated SQLite tests and shared all-state storage-format test pass. No Docker or network is used by these tests.
- Adapter tests prove reconnect, idempotent migration, queued/running/completed/rejected/failed readback, durable/competing IDs, stale/configuration rejection, event immutability, rollback on forced append failure, corrupt-state rejection, and unsupported-schema preservation. Temporary directories are test-owned and removed by tempfile.
- Strict adapter Clippy and `npm run validate:local` pass: 25 frontend tests, 34 Rust tests, three live Postgres tests, seven browser checks, formatting/contract/type/lint/build gates.
- rusqlite 0.40.2 bundles SQLite; tempfile resolved stable 3.27.0. Storage mapping moved unchanged into persistence-format; existing Postgres live checks still pass.
- Tauri path resolution and native process-restart proof belong to M5-02/03; this issue proves file-backed application persistence, not the installed UI.
