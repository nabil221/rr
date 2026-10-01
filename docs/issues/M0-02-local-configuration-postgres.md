# M0-02: Local Configuration and Postgres

**Status:** complete  
**Dependencies:** M0-01

## Objective

Make browser-server mode locally reproducible with project-scoped Postgres and explicit configuration.

## Scope

Add `.env.example`, server configuration validation, a local Postgres container definition, and safe lifecycle documentation. Do not add hosted deployment, credentials, or desktop database work.

## Implementation plan

1. Define local-only configuration values and validate them at server startup.
2. Add a named, project-scoped Postgres volume and container configuration.
3. Document start, stop, inspect, and reset actions with precise scope.
4. Ensure desktop defaults require no Postgres or network endpoint.

## Verification

- Start from an empty local database.
- Confirm invalid configuration prevents the server from listening.
- Confirm normal restart retains the named-volume data.

## Acceptance criteria

- Browser-server prerequisites start on a clean machine with documented commands.
- No secret is committed or required.
- Reset behavior cannot target databases outside this project.

## Evidence

- At M0, local configuration included and validated `DATABASE_URL` for the originally planned API startup. M1-04 intentionally superseded that runtime requirement: the current API binds loopback only and uses in-memory storage so the browser-to-backend stack is proven before introducing persistence. The local Postgres container remains available for the later persistence milestone and is not required to run M1.

- Historical M0 verification: `cargo test --workspace` passed, including the then-current API configuration validation for missing `DATABASE_URL`. That runtime configuration was removed for M1-04 so the API-only vertical slice does not require Postgres; see the M1-04 evidence.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo run -p local-stack-proof-api` correctly rejected missing `DATABASE_URL` with exit code 2.
- With the documented local `DATABASE_URL`, the API accepted configuration and reported `web-local` at `127.0.0.1:3000`.
- `docker compose -f infra/compose.yaml config` passed.
- PostgreSQL 18 Alpine started healthy on `127.0.0.1:54329` using volume `local_stack_proof_postgres_data`.
- A temporary database marker survived a container restart and was then removed.
