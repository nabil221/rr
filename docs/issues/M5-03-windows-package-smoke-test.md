# M5-03: Windows Package Smoke Test

**Status:** complete
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

The repeatable procedure is [desktop testing](../desktop-testing.md). NSIS uses
current-user installation and skips runtime installation; existing WebView2 is
required. The script retains isolated install/data folders and supplies a read-only
exact-ID verifier. Installation/uninstall registration remains normal NSIS behavior.

- Build installer/package.
- Stop browser API and Postgres.
- Install and launch packaged application.
- Create, execute, restart, and retrieve a run.

## Acceptance criteria

- Packaged desktop proof passes with no browser-server process.
- Installer prerequisites and data-retention behavior are documented.
- Permissions are limited to what the proof actually needs.

## Evidence

- `npm run package:desktop` builds the unsigned x64 current-user NSIS package (2,840,038 bytes). SHA256: `B8D052AF6AE9166AEF5BE8A3F3F6E65B4C7576265ABD09796B639AC08DBB5258`. Tauri fetched and hash-validated NSIS 3.11 and its utility plugin during the build; runtime installation is skipped, requiring existing WebView2.
- The smoke script installs into fresh `target/desktop-smoke/m5-package-installed-20260930`; app data is isolated in `target/desktop-smoke/m5-package-data-20260930`. Actual installed executable PID 5876 creates/completes run-00000000000000000001 and creates/rejects run-00000000000000000002 through the shared UI. Configuration/result/timelines and desktop-local/embedded-protocol/sqlite/ok diagnostics are visually verified.
- A confirmed process exit followed by installed executable PID 12892 retrieves both exact IDs, completed 24/7/64.14 metrics, and rejected validation/three-event history. `scripts/desktop-smoke.ps1 -Step Verify` passes read-only exact-ID/result/configuration/timeline verification afterward.
- Project API/Vite ports 3000/3001/5173/5174 and Postgres 54329 are closed during the installed test. No installed-app TCP socket is observed. Machine-wide networking is not disabled and WebView2 background behavior is not claimed.
- Local log inspection shows startup and safe method/route-template/status/timing, no IDs/payloads/credentials/paths. Capped append-only logging and read-only DB behavior pass targeted tests. A Windows append-only-handle test setup failure was corrected by separately sizing the test fixture; production permissions were not broadened. Strict native/SQLite Clippy passes.
- `scripts/desktop-smoke.ps1` syntax passes. Install/data artifacts are retained; no ordinary data/volumes were deleted. Uninstall/data-removal prompts and a clean machine without WebView2 are not tested; signing/public distribution remain excluded.
- Final `npm run validate:local` passes: 38 Rust tests, 25 frontend tests, three explicit live Postgres tests, seven browser workflows, contract/type/lint/format checks, and workspace/web builds. Project Postgres is restored and healthy; native app and test servers are closed. M5 is complete; stop before M6 authorization.
