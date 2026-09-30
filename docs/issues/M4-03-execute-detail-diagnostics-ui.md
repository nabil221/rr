# M4-03: Execute, Detail, and Diagnostics UI

**Status:** complete  
**Dependencies:** M4-02

## Objective

Make run execution observable and make active host/storage mode inspectable.

## Scope

Build execute action, polling status refresh, configuration snapshot, event timeline, results/validation states, and safe diagnostics. Exclude WebSockets and PWA caching.

## Implementation plan

1. Fetch detail by route ID, not a list-cache snapshot. Render configuration,
   explicit queued/running/completed/rejected/failed states, results and ordered events.
2. Execute only queued runs through the injected client; disable duplicate requests.
   Poll non-terminal snapshots once per second, with no overlapping polling reads.
   Reject stale reads by event-count monotonicity. Stop after terminal state, failure,
   or navigation; cancellation stops waiting, not committed server computation.
3. Keep failures and business rejection separate. Show safe API diagnostics with
   manual retry; never retry mutations automatically or invent job recovery.
4. Implement Diagnostics through the adapter's allowlisted metadata, identifying
   web-local/Postgres and desktop-local/in-memory. Health is liveness, not readiness.
5. Add targeted local browser execution/rejection/reload/diagnostics checks and
   fake-client/fake-time component checks for polling stop and cleanup.
6. Run validation and browser checks sequentially to avoid Windows executable locks;
   document actual evidence and commit before M4-04.

## Verification

- Complete a run without manual refresh.
- Run controlled rejection and confirm it differs from a zero-value result.
- Confirm Diagnostics identifies web-local/Postgres.

## Acceptance criteria

- Detail view can recreate a prior persisted run after refresh.
- Polling stops in terminal state.
- No secrets, paths, or stack traces appear in Diagnostics.

## Evidence

- `npm run validate` passed: 18 frontend tests, 29 Rust tests, contract/type/format/
  lint gates, and web/both host builds. Removed a helper export to preserve Fast
  Refresh's component-only boundary; targeted lint then passed without warnings.
- Five component tests prove queued/running/terminal observation, terminal polling
  stop, cancellation/timer cleanup on unmount, protection from a delayed stale
  read, stopped/retryable read errors, zero-valued result rendering, and injected
  native diagnostics (some tests cover multiple assertions).
- `npm run test:e2e -- -- detail.spec.ts` passed all three real-browser checks:
  completed/rejected execution, configuration and three events, no repeat-execute
  button, direct detail reload readback, and web-local/Postgres diagnostics.
- Mutations are not automatically retried. Client abort cancels waiting, not
  server commits; interrupted running jobs still have no recovery mechanism.
- Native diagnostics are tested with an injected client, not claimed as a new
  packaged native walkthrough. Desktop remains in-memory until M5.
