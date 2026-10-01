# M4-02: Create and List UI

**Status:** complete  
**Dependencies:** M4-01

## Objective

Let a browser user submit a configuration and revisit persisted run history.

## Scope

Build New Run form, server-validation display, run list, and navigation to detail. Exclude execution display, diagnostics, and PWA work.

## Implementation plan

1. Replace the New Run scaffold with a generated-type configuration form.
   Preserve raw input strings on failure; show the safe API message/code/field.
2. Create only a queued run; navigate to its returned ID without executing.
   Disable duplicate submissions and cancel client waiting on route exit.
3. Load history from the injected client with loading, empty, failure, retry/refresh
   states and encoded detail links. Do not use local storage as an alternative store.
4. Add targeted local browser tests for validation/input preservation, queued
   creation, refresh readback, and existing-row navigation. Leave execution to M4-03.
5. Validate, document actual evidence, and commit before M4-03.

## Verification

- Submit valid and rejected configurations against local API.
- Refresh list and confirm persisted records remain.

## Acceptance criteria

- Invalid submission preserves user input and explains error.
- Valid submission creates a queued persisted run.
- The UI uses contract-generated types/client.

## Evidence

- `npm run test:e2e -- -- create-list.spec.ts` passed against local API/Postgres:
  whitespace region returns 400, raw seed/region inputs remain, field is marked
  invalid, correction creates a queued run with one event, list refresh retains
  its row, and its encoded detail link navigates correctly.
- `npm run validate` passed (13 frontend tests, 29 Rust tests, all quality gates/builds).
- Initial lint found a synchronous state reset inside the load effect; moved
  refresh-state changes to the triggering event. No lint rule was disabled.
- Browser/aggregate checks are sequential after an initial overlap locked the
  running Windows API executable. Tests leave their queued proof run in the
  project database and do not delete existing rows or reset volumes.
