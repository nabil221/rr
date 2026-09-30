# Proof Matrix

Update this document as issues are completed. A claim is not proven until it has a repeatable local check or a documented demo observation.

| Claim                               | In-memory web          | In-memory desktop      | Browser/Postgres | Persistent desktop | Evidence / command                                   | Status       |
| ----------------------------------- | ---------------------- | ---------------------- | ---------------- | ------------------ | ---------------------------------------------------- | ------------ |
| Workspace quality gates             | proven                 | proven                 | proven           | proven             | `npm run validate`; DET-001                          | proven       |
| Shared Rust domain/application code | proven                 | proven                 | proven via HTTP  | pending            | M2-02; M3-01/M3-02 shared service + Postgres         | partial      |
| Frontend-to-backend interaction     | proven in Chrome UI    | proven in native UI    | pending          | pending            | M1-04 Chrome; M2-04 production Tauri UI walkthrough  | partial      |
| Create, execute, and read run       | proven                 | proven                 | proven via HTTP  | pending            | M1-04; M2-04; M3-02 real HTTP success/readback       | partial      |
| Controlled validation failure       | proven                 | proven                 | proven via HTTP  | pending            | M1-04; M2-04; M3-02 real HTTP rejection              | partial      |
| Persistence after restart           | n/a (in-memory)        | n/a (in-memory)        | API proven       | pending            | M3-01 reconnect; M3-02 API process restart readback  | partial      |
| HTTP contract/type alignment        | proven shared contract | proven shared contract | proven           | n/a                | M3-03 generated DTOs; drift experiment; validate     | proven       |
| Desktop works without API/Postgres  | n/a                    | proven                 | n/a              | pending            | M2-02 spike and M2-04 native UI; API/Vite/DB stopped | partial      |
| Windows packaged-build smoke path   | n/a                    | n/a                    | n/a              | pending            |                                                      | pending      |
| PWA installability, optional        | n/a                    | n/a                    | pending          | n/a                |                                                      | not selected |

## M3 adapter evidence

M3-01 is complete: three explicit live Postgres integration tests pass for
application-service create/execute/list/detail, reconnect readback, persistent IDs,
immutable event history, competing-write rejection, and atomic rollback.
`npm run validate` passes; ordinary tests skip the three explicit database tests.
No generated test schemas remained after execution. The HTTP host and process
restart are now proven by M3-02 below; desktop SQLite remains pending M5.

## M3 web-host evidence

M3-02 is complete: real loopback HTTP requests create/execute/read completed and
rejected runs through Postgres. Both results and timelines survive an API process
restart. Five shared-router tests cover safe failures, origin restrictions, request
IDs, limits, and lifecycle semantics; native in-memory protocol tests still pass.
`npm run validate` passes (27 Rust tests and 5 client tests). This is HTTP host proof,
not a new browser/native UI walkthrough; that remains the M4 UI verification scope.

## M3 contract evidence

M3-03 is complete: Rust Serde DTOs generate the committed TypeScript contract,
imported by the shared web/native client. An incompatible temporary Rust field rename
failed the check; reverting it restored a pass. Aggregate validation includes
contract checking before TypeScript and passes with 29 Rust tests and 5 client tests.
The three live database tests also pass. No runtime JSON validator or OpenAPI SDK
is claimed. All M3 issues are complete; execution stops before M4.

## M4 shell evidence

M4-01 and DET-002 are complete: injectable generated-type client, local-only
endpoint selection, four hash routes and shared responsive shell. Aggregate validation
passes (13 frontend tests, 29 Rust tests); the local headless browser smoke verifies
all routes, navigation, refresh, and no overflow at 1120px/360px. No new native
walkthrough is claimed. Create/list/detail workflows remain the next M4 issues.

## M4 create/list evidence

M4-02 is complete: real-browser server validation preserves input, valid creation
persists a queued run, and history readback survives refresh with detail navigation.
The targeted local E2E test and full aggregate validation passed. Execution and
safe host diagnostics remain M4-03; no client-side computation or alternate store.

## M4 detail/diagnostics evidence

M4-03 is complete: live browser completed/rejected execution, saved configuration,
ordered timeline, refresh readback and web-local/Postgres diagnostics all pass.
Five component tests prove polling stop/cleanup/stale-read protection and safe native
diagnostics injection. Aggregate validation passes (18 frontend/29 Rust tests).
Desktop transport tests remain passing; a new packaged native walkthrough is not claimed.

## Explicitly not proven

- Deployment, remote/networked access, accounts, synchronization, mobile support, real file ingestion, real Excel/workflow functionality, job durability beyond this local process model, and generic workflow authoring.
