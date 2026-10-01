# M2-03: Fallback Desktop Transport

**Status:** complete (conditional issue skipped; M2-02 passed)  
**Dependencies:** M2-02 failure

## Objective

Provide a narrow, testable Tauri-command transport while preserving the same Rust application services and host-neutral React UI.

## Scope

Implement only the command mapping and client adapter required by the spike; do not duplicate domain or application logic.

## Verification

- Call create-run through the command adapter in development and packaged builds.
- Compare DTO shape and error behavior with HTTP contract expectations.

## Acceptance criteria

- React screens do not contain Tauri-specific business branches.
- ADR-001 documents the router rejection and fallback evidence.

## Evidence

The embedded-router spike passed in development and production. ADR-001 accepts that path, so no command fallback is implemented. Reopen this conditional issue only if new reproducible evidence invalidates the selected transport.
