# M3-02: Axum API

**Status:** planned  
**Dependencies:** M3-01, M1-04

## Objective

Replace the M1-04 in-memory host wiring with the Postgres-backed application composition while preserving and hardening the proven HTTP API.

## Scope

Finalize health, create, execute, list, and detail routes; transport DTOs; request IDs; local CORS; body limits; logs; and safe error envelope around the M1-04 slice. Exclude auth, remote access, streaming, and business logic in handlers.

## Verification

- Router tests cover success, validation error, malformed payload, and unknown run.
- Verify CORS permits only intended local UI origin.
- Inspect logs for correlation ID and no sensitive error leakage.

## Acceptance criteria

- HTTP validation returns useful 4xx response content.
- Unexpected failure returns a safe 5xx envelope.
- Handlers delegate all calculation to application services.

## Evidence

_Pending execution._
