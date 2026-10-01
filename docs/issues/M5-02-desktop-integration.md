# M5-02: Desktop Integration

**Status:** complete
**Dependencies:** M5-01, M4-01, whichever desktop transport path (M2-02 or M2-03) is selected

## Objective

Run the shared React screens and shared Rust application services as a local Tauri desktop application.

## Scope

Implement selected transport client, Tauri host configuration, production React bundle integration, minimal local permissions, and desktop diagnostics. Exclude installer testing.

## Implementation plan

1. Compose the SQLite repository during Tauri setup at the application-local-data directory; fail safely rather than reverting to memory. Provide an explicit absolute test-data-directory override.
2. Keep the existing custom protocol, limits, exact-origin boundary, and shared client/screens. Label native health storage as SQLite.
3. Add process-free protocol tests against isolated SQLite, and run the actual native development UI through creation, success/rejection, history, detail refresh, and diagnostics.
4. Rebuild embedded production assets and verify persistence across host restart. Document local data and interrupted-running behavior without adding recovery jobs or synchronization.

## Verification

- Use Runs, New Run, Detail, and Diagnostics in desktop development mode.
- Confirm desktop uses SQLite and no external API process.

## Acceptance criteria

- Screen components have no desktop-specific business branches.
- Diagnostics identifies desktop-local/SQLite.
- The desktop build starts without a network connection or Node server runtime.

## Evidence

- Implementation is wired to per-user SQLite with an absolute local test-directory override; no React host-specific business branch or backend listener was added. Native health labels SQLite.
- `cargo test -p local-stack-proof-desktop`: six tests pass, including local path selection/reconnect and native SQLite create/execute/rejection/detail/history after host recomposition. Strict desktop Clippy passes.
- The user signaled native-test readiness. Actual development UI walkthrough passes with isolated `target/desktop-smoke/m5-dev-20260930/runs.sqlite3`: empty history, queued creation, rejected run-00000000000000000001, completed run-00000000000000000002, saved configuration, three-event timelines, result 24/7/64.14, detail reload, history, and desktop-local/embedded-protocol/sqlite/ok diagnostics.
- After a real development-host close/relaunch, both exact IDs remain and the completed result is retrieved through the UI. Computer-use accessibility click coordinates were unreliable on this window; screenshot-backed coordinates were used and visually verified instead. No application workaround was introduced.
- `npm run build:desktop` passes. The freshly rebuilt production executable starts with embedded assets and reads both exact development-created IDs; the completed result/configuration/three-event timeline and SQLite diagnostics are visually verified.
- Production test ran with ports 3000/3001/5173/5174/54329 closed and project Postgres exited. The desktop process owns no TCP connection/listener at the inspected checkpoint. The machine's network adapter was not disabled; this proves no project-server/runtime dependency, not a claim about all possible WebView2 background traffic.
- Existing aggregate regression evidence is M5-01; desktop's six tests, strict Clippy, frontend 25 tests, typecheck/lint/format checks pass after wiring. Windows installation remains M5-03.
