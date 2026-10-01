# M3-01: Postgres Persistence

**Status:** complete  
**Dependencies:** M1-03, M0-02

## Objective

Implement a local Postgres adapter that replaces the in-memory adapter and preserves runs and their event history for browser-server mode.

## Scope

Create migrations, repository implementation, and isolated integration tests. Exclude API routes and UI.

## Implementation plan

1. Define tables for runs and immutable run events.
2. Store configuration/result payloads only as needed to recreate detail view.
3. Add migrations and Postgres repository mapping.
4. Add integration tests using project-scoped test data.

Concrete design is recorded before implementation in [ADR-002](../decisions/0002-local-postgres-persistence.md). Keep the synchronous application port; the HTTP adapter will dispatch service operations on blocking workers in M3-02. Production storage is confined to the `stack_proof` schema of the `local_stack_proof` database. Integration tests use unique `stack_proof_test_*` schemas, never public or production schema. Repositories append lifecycle events transactionally, reject stale saves, reconstruct through existing domain transitions, and allocate IDs from a database sequence so restart cannot collide.

## Verification

- Create, execute, list, and retrieve a run through application services.
- Restart the adapter/server and confirm retained state.

## Acceptance criteria

- No test targets a developer's default database schema.
- Database errors map to safe application errors.
- Schema is deliberately proof-sized, not product-scale.

## Evidence

- `npm run validate` passed: formatting, TypeScript, Oxlint, workspace Clippy,
  23 Rust unit tests, 5 web client tests, and web/all Rust host builds.
- `cargo test -p local-stack-proof-persistence-postgres -- --ignored` passed all
  3 live database tests: completed/rejected run reconnect and ID uniqueness;
  stale writes, immutable events, corrupt-state rejection and atomic rollback;
  competing transitions across two independent connections.
- Targeted application/persistence unit tests passed (8 tests), including all
  five lifecycle states and safe local-only configuration validation.
- A Docker/psql query of `information_schema.schemata` returned no
  `stack_proof_test_*` schemas after testing. Existing application data was not reset.
- The synchronous `postgres` 0.19.14 adapter implements the existing repository
  port. Persistent ID allocation can return repository errors; in-memory saves
  now enforce the same append-one-event conflict rule.
- This proves adapter reconnect, not yet HTTP server restart. Web host wiring is
  M3-02; desktop remains in-memory. No pool, job recovery, or throughput claim.
