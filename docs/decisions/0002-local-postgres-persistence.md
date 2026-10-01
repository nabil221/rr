# ADR-002: Local Postgres Persistence

**Status:** accepted and verified in M3-01

## Context

M1/M2 already prove both transports. M3 adds persistence only to the local web host. Application repository ports are synchronous, the domain has no serialization/framework dependencies, and desktop must remain database-free until its SQLite milestone.

## Options considered

1. A synchronous Postgres client behind the existing repository port, with blocking-worker dispatch at asynchronous host boundaries.
2. Convert all application ports and both hosts to async to introduce an async SQL driver.

## Decision

Use option 1. A single mutex-protected client is sufficient for the proof's tiny local workload; no throughput or pooling claim is made. Keep SQL and storage serialization inside a new `persistence-postgres` crate. Add explicit connect timeouts and local-only URL/database validation. Do not print connection strings or database error details.

Use project-owned `stack_proof` schema and versioned SQL migration. Store a JSONB run snapshot plus append-only, ordered event rows. Snapshot and new event commit in one transaction. Lock the run row and require exactly one appended lifecycle event with an unchanged prefix/configuration: stale saves return a safe conflict. This prevents competing execution requests from overwriting one another. IDs come from a schema-local database sequence and remain unique after restart.

Reconstruct saved runs using the existing domain lifecycle methods, then compare reconstructed snapshot/events with stored values. Invalid stored state is a safe repository error, not silently repaired or recalculated.

Integration tests create unique `stack_proof_test_*` schemas in the project database, run migrations twice, exercise the real application service, reconnect, test event immutability and stale writes, and clean up only their validated schema. No test targets public or production schema. Ordinary unit tests do not need Docker; a separate explicit integration command requires the project database and fails if it is absent.

## Evidence

- The user authorized M3. The project Postgres container is running and healthy.
- Current stable dependency versions and driver behavior are verified against registry and official crate documentation before installation.
- All three live integration tests and aggregate workspace validation passed;
  [M3-01](../issues/M3-01-postgres-persistence.md) records the evidence and limitations.

## Consequences

Postgres-backed HTTP service calls must use blocking workers rather than blocking the async executor. Desktop continues composing the in-memory service. A process interruption after the committed STARTED transition can leave a run running; durable job recovery/automatic retry is outside this proof and is documented, not hidden.

## Rollback/fallback

Keep existing data/volumes. If live Postgres verification fails, mark M3-01 incomplete and diagnose the local prerequisite or adapter; do not fall back silently to in-memory or change databases. Desktop remains available independently.
