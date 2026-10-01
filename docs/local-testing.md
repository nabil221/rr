# Local verification

## Ordinary quality gates

`npm run validate` checks formatting, Rust-generated TypeScript contract, typecheck,
Oxlint, Clippy, frontend/component and Rust unit tests, then web/Rust builds.
It does not require Docker or a browser. Component tests use injected clients;
polling checks use fake time rather than real sleeps.

## Full local proof

Start Docker and the project database, then run:

```text
docker compose -f infra/compose.yaml up -d --wait
npm exec -w @local-stack-proof/web -- playwright install chromium
npm run validate:local
```

Browser installation is a one-time prerequisite after installing/upgrading Playwright.
`validate:local` runs ordinary validation, the three explicit Postgres adapter tests,
then browser automation sequentially. Do not run builds concurrently with browser
tests: Windows cannot replace the API executable while its test process is running.

For targeted checks:

```text
npm run test:postgres
npm run test:e2e
npm run test:e2e -- -- detail.spec.ts
npm run test:e2e -- -- --repeat-each=2
```

The browser harness starts its own loopback API on 3001 and UI on 5173. Stop your
development UI before testing; occupied ports fail rather than reuse or terminate
unknown processes. Port 3000 is not touched. The UI uses an explicit test-only
loopback endpoint; CORS still permits only the intended 5173 origin.
Playwright waits for server readiness and tests wait on DOM/HTTP state, not arbitrary
timeouts. Fresh headless browser contexts never access your Chrome profile.
The harness closes the processes it owns after execution.

## Data and evidence

Adapter tests create and clean unique `stack_proof_test_*` schemas. Browser tests
exercise the real application's `stack_proof` schema and append small demo runs.
The proof has no delete API, so these runs remain visible for inspection; tests do
not reset volumes, delete existing records, or assume an initially empty database.
Empty/failure states are checked with injected clients or a narrowly mocked request.

Named tests identify the failed stage. Failure screenshots/traces and desktop/narrow
completed-detail screenshots are under `apps/web/test-results/` (git-ignored).
Open a failure trace using the project Playwright CLI; never publish traces containing
local run data without review. Screenshots are local verification artifacts, not a
pixel-perfect visual-regression contract.

## Local endpoint override

Web defaults to the same-origin Vite `/api` proxy targeting port 3000. Optionally set
`VITE_API_BASE_URL` before starting Vite or in `apps/web/.env.local` (git-ignored).
Only credential-free loopback HTTP roots are accepted, with no path/query/fragment.
Malformed settings produce a safe configuration screen. Native ignores this override
and always uses the embedded protocol. UI code does not branch on host identity.

The native development CLI overrides Vite to port 5174; the web default is strict 5173
to match its exact-origin policy. A new native packaged walkthrough belongs to M5;
headless browser/component checks do not substitute for that acceptance evidence.

For SQLite/native tests and actual Windows installer verification, see
[desktop testing](desktop-testing.md). SQLite tests need no Docker; the installed
package smoke deliberately runs with project servers stopped, then restores Postgres
before the full web regression suite. Installation and run data are kept separate.
