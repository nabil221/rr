# Proof Matrix

Update this document as issues are completed. A claim is not proven until it has a repeatable local check or a documented demo observation.

| Claim                               | In-memory web          | In-memory desktop      | Browser/Postgres  | Persistent desktop   | Evidence / command                               | Status       |
| ----------------------------------- | ---------------------- | ---------------------- | ----------------- | -------------------- | ------------------------------------------------ | ------------ |
| Workspace quality gates             | proven                 | proven                 | proven            | proven               | `npm run validate:local`; DET-001/002; M5-03     | proven       |
| Shared Rust domain/application code | proven                 | proven                 | proven in browser | proven in native UI  | M2-02; M3 persistence; M4/M5 UI flows            | proven       |
| Frontend-to-backend interaction     | proven in Chrome UI    | M2 native baseline     | proven in browser | proven in native UI  | M1-04; M2-04; M4-04; M5-02/03                    | proven       |
| Create, execute, and read run       | proven                 | M2 native baseline     | proven in browser | proven in native UI  | M4-02/03/04; M5-02/03 UI workflows               | proven       |
| Controlled validation failure       | proven                 | M2 native baseline     | proven in browser | proven in native UI  | M4-04; M5-02/03 rejection/readback               | proven       |
| Persistence after restart           | n/a (in-memory)        | n/a (in-memory)        | API proven        | installed app proven | M3-01/02; M5-02/03 process restart readback      | proven       |
| HTTP contract/type alignment        | proven shared contract | proven shared contract | proven            | n/a                  | M3-03 generated DTOs; drift experiment; validate | proven       |
| Desktop works without API/Postgres  | n/a                    | proven                 | n/a               | installed app proven | M5-03 installed UI; API/Vite/DB stopped          | proven       |
| Windows packaged-build smoke path   | n/a                    | n/a                    | n/a               | installed app proven | M5-03; `npm run package:desktop`; smoke script   | proven       |
| PWA installability, optional        | n/a                    | n/a                    | pending           | n/a                  |                                                  | not selected |

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
is claimed. All M3 issues were completed before M4 authorization.

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

## M4 browser automation evidence

M4-04 is complete. Final `npm run validate:local` passes: 25 frontend tests,
29 Rust unit tests, three explicit Postgres tests, seven browser workflows,
and all quality/build gates. A repeat-each=2 browser run passes all 14 checks
with zero retries. Desktop/narrow screenshots were inspected; no document
overflow is observed. Native diagnostics/client routing are tested through
injection; native protocol tests pass, but the new M4 native UI/SQLite/package
walkthrough remains M5. Browser test records are retained in project storage.
All four M4 issues are complete; stop before M5 pending user confirmation.

## M5 adapter evidence

M5-01 is complete: bundled SQLite behind the shared repository ports passes five
isolated file-backed tests, including reconnect, persistent IDs, competing writes,
immutable events, rollback, and corrupt/unsupported state. Shared serialization
passes all lifecycle roundtrips; the Postgres/browser regression suite still passes.
`npm run validate:local`: 25 frontend, 34 Rust, three Postgres, seven browser tests.
Native composition and installed-package UI proof remain M5-02/03.

## M5 native integration evidence

M5-02 is complete: actual development UI creates/executes completed and rejected
runs, renders saved results/events and native SQLite diagnostics, and retrieves the
exact IDs after process restart. Production embedded assets start and read the same
SQLite file with API/Vite/Postgres stopped. No desktop-owned TCP socket is observed.
Machine-wide networking was not disabled. Actual Windows installation remains M5-03.

## M5 installed-package evidence

M5-03 is complete. The actual current-user NSIS installer installs into an isolated
project directory. Its installed executable creates/executes completed and rejected
runs through the shared UI, then retrieves both exact IDs, results/configuration,
and three-event histories after confirmed process exit/relaunch. The read-only
`scripts/desktop-smoke.ps1 -Step Verify` check passes. Installer hash and exact paths
are recorded in [M5-03](issues/M5-03-windows-package-smoke-test.md).

API/Vite/Postgres were stopped; no installed-app TCP socket was observed. Machine
networking was not disabled. Existing WebView2 is required; missing-runtime clean
machines, signing/distribution, and uninstall prompts were not tested. Safe capped
local logging and explicitly empty frontend plugin capabilities are verified.
Isolated installation/data remain available; ordinary data and Docker volume were
preserved. Postgres is restored and healthy.

Final `npm run validate:local` passes: 38 Rust tests, 25 frontend tests, three live
Postgres tests, seven browser workflows, and all contract/type/lint/build gates.
M5 is complete. Stop before M6, which requires user confirmation.

## Explicitly not proven

- Deployment, remote/networked access, accounts, synchronization, mobile support, real file ingestion, real Excel/workflow functionality, job durability beyond this local process model, and generic workflow authoring.
