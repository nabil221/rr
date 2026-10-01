# M2-02: Embedded Desktop Transport Spike

**Status:** complete  
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

## Evidence

- Initial development startup on Tauri's conventional port 1420 failed with Windows `EACCES`. Read-only `netsh interface ipv4 show excludedportrange protocol=tcp` confirmed that 1420 lies in the excluded 1332-1431 range. Selected available port 5174 for desktop Vite and updated the exact development origin/CSP. This is an environment configuration correction, not a custom-protocol failure.
- Development and production WebView2 harnesses passed health (200), create (201), invalid configuration (400 with code/field), and missing ID (404 with code). A final health request is issued only after JavaScript validates every preceding response body; both hosts reached that checkpoint.
- Production evidence: `target/m2-evidence/spike-production-20260930-112129.stderr.log`. API, both Vite ports (5173/5174), and Postgres were stopped; process 22376 had no listening TCP socket. These are local generated logs, not source-controlled artifacts.
- Shared-router tests cover success, execute/read/list, duplicate execution, rejection, invalid input, and missing IDs. Four desktop adapter tests cover health, preflight, untrusted origins, and body bounds. Five frontend client tests cover web/native URLs, error metadata, and AbortSignal forwarding.
- Full native React interaction remains a separate M2-04 acceptance check; the harness does not claim that UI walkthrough.
