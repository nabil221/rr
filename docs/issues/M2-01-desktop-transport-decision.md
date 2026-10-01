# M2-01: Desktop Transport Decision

**Status:** complete  
**Dependencies:** M1-04

## Objective

Define an evidence-based choice between embedded Axum routing in Tauri and a narrow Tauri-command adapter.

## Scope

Create ADR-001 and a time-boxed spike plan. Preferred option is embedded Axum; fallback is commands calling the same application services. Do not build desktop UI.

## Implementation plan

1. Write pass/fail criteria: packaged Windows behavior, typed errors, cancellation, progress, logs, testability, and React divergence.
2. Define the smallest endpoint/use-case needed for the spike.
3. Specify the fallback and its non-negotiable shared-core requirement.

## Verification

- Review ADR criteria before M2-02 begins.

## Acceptance criteria

- Criteria are objective enough to select or reject the preferred path.
- The spike has a bounded scope and fallback.
- No future UI issue assumes an unproven transport.

## Evidence

- [ADR-001](../decisions/0001-desktop-transport.md) defines the preferred asynchronous custom-protocol -> shared Axum router path and the command-adapter fallback.
- Criteria cover development/production WebView2, error status/JSON, strict origins, process-free adapter tests, request logs, no listening sockets, and shared frontend/application code.
- Cancellation is explicitly limited to aborting the fetch wait; progress uses the existing lifecycle/status contract. Neither introduces unrelated business functionality.
- The bounded spike must precede connecting the React desktop flow; installer work remains in M5. Stable dependency versions were verified before implementation.
