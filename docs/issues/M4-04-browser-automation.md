# M4-04: Browser Automation

**Status:** complete  
**Dependencies:** M4-03

## Objective

Automate the main browser proof paths against local processes.

## Scope

Add component tests plus browser automation for create, execute, inspect, rejection, and refresh persistence. Exclude remote test services.

## Implementation plan

1. Extend DET-002's local harness rather than introducing another runner.
   Keep explicit `test:e2e` separate from ordinary validation; add an opt-in
   `validate:local` aggregate covering unit/build, database, then browser tests.
2. Add component coverage for form input preservation, duplicate-submission
   prevention, empty/list/retry states, rejection vs zero-result rendering,
   polling stop/cleanup/stale reads, and native diagnostics via an injected client.
3. Consolidate real-browser checks for create/list, execute/result/timeline,
   controlled rejection, direct-link reload, safe diagnostics, external execution
   observed through polling, and retry after a simulated transport failure.
4. Use semantic locators, awaited HTTP/DOM state, and fake time where appropriate;
   never use sleep-based assertions. Name tests/stages and retain failure traces.
5. Preserve existing project database rows; tests append small recognizable proof
   runs because there is intentionally no delete API. Document this limitation.
6. Inspect desktop/narrow browser screenshots, repeat the browser suite, run all
   checks sequentially, update the proof matrix, and stop before M5.
   No new native packaging or SQLite claim; native UI verification remains M5.

## Verification

- Run automation against locally started API/UI/database.
- Confirm tests wait on UI/API state rather than arbitrary timeouts.

## Acceptance criteria

- Happy and rejected end-to-end paths pass reproducibly.
- Failure output identifies the failed stage clearly.

## Evidence

- `npm run validate:local` passed twice, including the final navigation-boundary
  change: 25 frontend tests, 29 Rust unit tests, all format/contract/type/lint/build
  gates, three live Postgres tests, and seven real-browser scenarios.
- `npm run test:e2e -- -- --repeat-each=2` passed all 14 checks in 33.1 seconds
  with zero retries. Real workflows cover input validation/queued creation,
  persisted history, success/rejection execution, configuration/events, detail
  reload, safe web diagnostics, external execution observed by polling, explicit
  transport-failure retry, and desktop/narrow shell/navigation.
- Component coverage adds duplicate-submission prevention, empty/history retry,
  safe unexpected errors, terminal/failed/rejected/zero-result distinctions,
  polling stop/cancellation/stale-read protection, native diagnostics injection,
  and safe invalid-endpoint configuration rendering.
- Inspected completed-detail screenshots at 1120px/360px: navigation, configuration,
  metrics/table, events, and long IDs remain readable without horizontal overflow.
  Screenshots are local ignored artifacts, not a visual-regression baseline.
- The local harness owns its processes, refuses occupied ports, uses fresh browser
  contexts, and preserves existing project database records. Browser-created proof
  runs remain intentionally; no delete API or database reset was added.
- [Local testing guide](../local-testing.md) documents prerequisites, commands,
  process/port behavior, artifacts, endpoint overrides, and data limitations.
- M4 is complete. Stop before M5; no SQLite, installer, or new native packaged
  UI walkthrough is claimed by these browser/component checks.
