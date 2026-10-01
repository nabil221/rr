# M2-02: Embedded Desktop Transport Spike

**Status:** planned
**Dependencies:** M2-01

## Objective

Prove or reject a minimal embedded Axum request path inside a packaged Tauri application.

## Scope

Implement health and create-run calls through the selected embedded-router mechanism. Exclude full UI, SQLite, and feature polish.

## Implementation plan

The concrete transport mechanism and pass/fail limits are recorded before implementation in [ADR-001](../decisions/0001-desktop-transport.md). Production testing here means the Windows executable with embedded frontend assets, not an installer.

1. Compose the existing application service in a Tauri host.
2. Register health and create-run routes before WebView startup.
3. Call them from a minimal frontend/test harness.
4. Test development and packaged Windows behavior.
5. Record result in ADR-001.

## Verification

- Run without Axum API executable or Postgres.
- Verify error propagation and logs.
- Verify no externally reachable public server port is bound.

## Acceptance criteria

- All ADR pass criteria have evidence, or a specific failure triggers M2-03.
- The shared application service, not duplicated logic, handles the request.


