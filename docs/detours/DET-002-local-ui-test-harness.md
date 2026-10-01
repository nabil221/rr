# DET-002: Local UI verification harness

**Status:** complete  
**Date:** 2026-09-30  
**Return point:** M4-01 verification; M4-04 remains planned.

## Why now

Chrome browser-client selection reported unavailable. Discovery found Edge and the
in-app browser, not Chrome; do not switch the user's chosen browser or modify profiles.
The M4 issues need interactive verification before issue completion. Waiting until
M4-04 to install the harness would make earlier acceptance evidence incomplete.

## Outcome and scope

Install project-scoped current stable Playwright Test, React Testing Library/DOM,
and jsdom. Configure isolated, headless Chromium tests with loopback-only UI/API
processes. Add only a shell/navigation/layout smoke test now, plus component setup.
The harness will be reused for each issue's targeted checks; M4-04 owns comprehensive
happy/rejection/refresh/failure scenarios. No user-profile access, remote test service,
product dependencies, SQL reset, SQLite, native packaging, or M5 work.

## Boundary and authorization

The user authorized M4, including local component/browser automation. This is an
early bootstrap of that same approved tooling, not a core-stack replacement or new
milestone. Registry checks: Playwright Test 1.63.0, React Testing Library 16.3.3,
DOM Testing Library 10.4.2, jsdom 30.1.1. Install browser binaries locally as test tools;
do not install extensions or repair native-host manifests.

## Verification and stop condition

- Unit/component runner still passes existing tests.
- Headless shell smoke renders all four routes and navigates at desktop/narrow widths.
- Test servers refuse port conflicts rather than stopping/reusing unknown processes.
- Artifacts are ignored; no existing database records are deleted.
- Close the detour when the smoke passes and return to M4-01 evidence/commit.
  If browser installation cannot be made to work, record the environmental blocker;
  do not claim unverified UI acceptance or advance the backlog.

## Evidence

- Registry-selected dependencies installed with zero audit findings. Chromium installed
  as a test tool; no user profile or extension was changed.
- `npm run test:e2e -- -- shell.spec.ts` passed: four routes at 1120px/360px,
  no document overflow, navigation, and deep-link refresh. Test servers shut down afterward.
- Aggregate checks passed with 13 client/shell tests and 29 Rust tests; lint/typecheck
  include the harness. Comprehensive workflow tests remain M4-04.
- Returned to M4-01 evidence/commit. The unavailable Chrome connection is not
  presented as a successful interactive Chrome check.
