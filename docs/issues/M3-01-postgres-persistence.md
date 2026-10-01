# M3-01: Postgres Persistence

**Status:** planned
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


