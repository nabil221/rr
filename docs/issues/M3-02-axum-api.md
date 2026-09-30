# M3-02: Axum API

**Status:** complete  
**Dependencies:** M3-01, M1-04

## Objective

Replace the M1-04 in-memory host wiring with the Postgres-backed application composition while preserving and hardening the proven HTTP API.

## Scope

Finalize health, create, execute, list, and detail routes; transport DTOs; request IDs; local CORS; body limits; logs; and safe error envelope around the M1-04 slice. Exclude auth, remote access, streaming, and business logic in handlers.

## Implementation plan

1. Compose the Postgres service synchronously before starting the API's Tokio runtime;
   retain an outer service owner so the synchronous driver's runtime is dropped outside async execution.
2. Dispatch shared service calls through `spawn_blocking`. Preserve the desktop router's
   in-memory composition and native origin policy; introduce a separate web-router wrapper.
3. Add a fixed 64 KiB JSON limit and a consistent safe JSON envelope for invalid JSON,
   missing routes, method errors, repository failures, and optimistic-write conflicts.
4. Generate server-owned request IDs, return them in response headers, and log only ID,
   method, route pattern, status, and duration (not payloads, URLs, credentials, or driver errors).
5. Permit web origins only at `http://127.0.0.1:5173`; reject other explicit origins before
   dispatch and handle exact-origin preflight. Origin checks are not authentication.
6. Test the router with in-memory and deliberately failing repositories; verify the real
   local HTTP/Postgres path and restart readback without resetting existing data.
7. Update current startup instructions and storage copy only; defer new UI functionality to M4.

## Verification

- Router tests cover success, validation error, malformed payload, and unknown run.
- Verify CORS permits only intended local UI origin.
- Inspect logs for correlation ID and no sensitive error leakage.

## Acceptance criteria

- HTTP validation returns useful 4xx response content.
- Unexpected failure returns a safe 5xx envelope.
- Handlers delegate all calculation to application services.

## Evidence

- `npm run validate` passed: all quality gates, 27 Rust unit tests,
  5 web tests, React production build, and both Rust hosts.
- Five shared-router tests cover lifecycle success/rejection, unknown runs,
  repeat execution, malformed/oversized JSON, missing routes/method errors,
  exact web origin/preflight, server-owned request IDs, and safe 500/409 envelopes.
- Three API config tests cover local defaults and rejection of non-loopback,
  empty, remote, or wrong database configuration without credential disclosure.
- Real loopback HTTP smoke test created and executed successful/rejected runs
  in `stack_proof`. Success returned 24 records, 7 matches, average 64.142857,
  and created/started/completed events. Rejection retained its message and three events.
- Stopped only the verified smoke-test API process and launched a new process.
  Both runs, their results/validation, and timelines were read back unchanged.
  The terminal's Ctrl+C did not reach the Windows child, so this smoke test used
  process termination rather than claiming graceful shutdown was manually verified.
- Inspected logs: server-generated IDs, methods, route patterns, status, duration;
  no request body, driver errors, URL, or database credentials.
- Desktop protocol tests still pass and report in-memory storage. New native/browser
  UI walkthroughs remain M4 work; no desktop Postgres dependency was introduced.
- Initial aggregate build hit the running smoke-test executable's Windows lock;
  stopping that process and rerunning the full command passed. Test runs remain
  in project storage for inspection; no existing data or volumes were reset.
