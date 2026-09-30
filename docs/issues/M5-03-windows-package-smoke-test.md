# M5-03: Windows Package Smoke Test

**Status:** in progress
**Dependencies:** M5-02

## Objective

Produce and validate the actual local Windows package, not only desktop development mode.

## Scope

Configure package metadata/icons/logs and create a repeatable local install/launch/create/execute/restart smoke script. Exclude public distribution and code signing unless packaging requires a local prerequisite.

## Implementation plan

1. Enable a current-user NSIS package with existing local icons and minimal capabilities. Use the installed WebView2 runtime as an explicit offline prerequisite; do not add signing or distribution infrastructure.
2. Add a repeatable local smoke preparation/verification script with explicit package/data paths and no database reset. Install the project-built package locally, preserving ordinary application data.
3. Stop only project-owned API/Vite processes and the project Postgres container for the test; launch the installed executable with isolated data and exercise the shared native UI.
4. Restart and retrieve the exact completed/rejected runs; inspect desktop-owned network listeners and document installer/data retention and limitations. Restore the project database afterward.

## Verification

- Build installer/package.
- Stop browser API and Postgres.
- Install and launch packaged application.
- Create, execute, restart, and retrieve a run.

## Acceptance criteria

- Packaged desktop proof passes with no browser-server process.
- Installer prerequisites and data-retention behavior are documented.
- Permissions are limited to what the proof actually needs.

## Evidence

_Pending execution._
