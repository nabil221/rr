# M5-02: Desktop Integration

**Status:** planned
**Dependencies:** M5-01, M4-01, whichever desktop transport path (M2-02 or M2-03) is selected

## Objective

Run the shared React screens and shared Rust application services as a local Tauri desktop application.

## Scope

Implement selected transport client, Tauri host configuration, production React bundle integration, minimal local permissions, and desktop diagnostics. Exclude installer testing.

## Verification

- Use Runs, New Run, Detail, and Diagnostics in desktop development mode.
- Confirm desktop uses SQLite and no external API process.

## Acceptance criteria

- Screen components have no desktop-specific business branches.
- Diagnostics identifies desktop-local/SQLite.
- The desktop build starts without a network connection or Node server runtime.


