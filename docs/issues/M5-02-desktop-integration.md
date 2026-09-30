# M5-02: Desktop Integration

**Status:** in progress
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
- Actual development/production UI walkthrough is pending. The user requested notification when UI testing is needed while currently using CLI; UI automation is held until they signal readiness. This issue remains in progress, not complete.
