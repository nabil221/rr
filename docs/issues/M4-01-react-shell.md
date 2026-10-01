# M4-01: React Shell and Client Boundary

**Status:** planned  
**Dependencies:** M3-03, whichever desktop transport path (M2-02 or M2-03) is selected

## Objective

Harden and expand the M1-04 proof UI into the host-neutral React shell used by the persistent browser and desktop flows.

## Scope

Refactor the thin M1-04 screen into routes for Runs, New Run, Detail, and Diagnostics; add loading/empty/error states; configurable API endpoint; and a client interface. Exclude full forms and desktop packaging.

## Verification

- Type-check and render each route.
- Confirm screen components depend on client interface, not direct browser or Tauri APIs.

## Acceptance criteria

- UI is usable at normal desktop widths and resilient at narrow widths.
- No domain calculation is reimplemented in React.
- Host identity is not scattered through UI code.

## Evidence

_Pending execution._
