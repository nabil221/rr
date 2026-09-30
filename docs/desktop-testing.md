# Desktop local verification

## Storage and boundaries

Desktop uses the embedded `proof-api` protocol and bundled SQLite; it does not
listen on a backend TCP port and does not connect to Postgres. The React screens,
HTTP DTOs, Rust domain, and application service are shared with web. Stores remain
independent, including their IDs; identical IDs in different stores are unrelated.

Default Windows data is under `%LOCALAPPDATA%/dev.localstackproof.desktop/`:
`runs.sqlite3` is the run database, and Tauri/WebView2 keeps its own browser cache.
An absolute local `LOCAL_STACK_PROOF_DATA_DIR` overrides **run data**, not WebView2
cache. Relative paths and Windows network shares are rejected. SQLite is bundled;
no system SQLite install, Docker, or API process is needed. Startup fails safely
if storage cannot be opened; it never falls back to volatile memory or resets data.

No automatic job recovery is promised: interruption after STARTED can leave a
persisted running run. Reload cancels only the client's wait, not committed work.

## Isolated development walkthrough

Set a unique absolute directory in the invoking shell, then run from repo root:

```powershell
$env:LOCAL_STACK_PROOF_DATA_DIR = Join-Path (Get-Location) 'target/desktop-smoke/dev'
npm run dev:desktop
```

Development requires Vite on 5174. That is a frontend dev-server prerequisite, not
a desktop backend socket. Check:

1. Runs starts empty in a new data directory.
2. New Run creates queued state; Execute produces completed metrics and three events.
3. Force controlled rejection, execute, and inspect validation and three events.
4. Runs, detail reload, and Diagnostics all work; diagnostics says desktop-local,
   embedded-protocol, sqlite, ok and exposes no paths/connection strings.
5. Close/relaunch against the same directory; both terminal runs still exist.

Remove the environment variable after testing using
`Remove-Item Env:LOCAL_STACK_PROOF_DATA_DIR`. This does not delete data. Keep test
data for inspection, or remove only its explicitly verified test directory later.

## Production executable

`npm run build:desktop` embeds the current production React assets into
`target/release/local-stack-proof-desktop.exe`. Stop Vite/API and verify the same
walkthrough with the production executable. M5-03 adds actual installer evidence;
a standalone executable or injected-client test is not installation proof.

## Automated checks

`cargo test -p local-stack-proof-persistence-sqlite` exercises file-backed repository
semantics in tempfile directories, without network/database services.
`cargo test -p local-stack-proof-desktop` checks native protocol limits/origins,
local path selection, SQLite contract, and host recomposition. Ordinary
`npm run validate` includes these tests; `validate:local` additionally checks web.

## Current evidence

M5-01/02/03 are complete: adapter/protocol checks, development success/rejection,
real host restart, and actual installed Windows-package creation/execution/restart
readback pass. Production and installed builds ran with project API/Vite/Postgres
stopped; no native-owned TCP socket was observed. Machine networking was not
disabled. The read-only exact-ID verifier and full local regression suite pass;
Postgres is restored. Package hash and observations are recorded in
[M5-03](issues/M5-03-windows-package-smoke-test.md). Stop before M6 authorization.

## Windows package smoke procedure

Build `npm run package:desktop`. The unsigned current-user NSIS installer is under
`target/release/bundle/nsis/`. Existing WebView2 is an explicit prerequisite:
`webviewInstallMode=skip` prevents downloading/installing a runtime during this
local offline proof. A clean machine without WebView2 is **not** supported by this
package yet. No signing, updater, cloud service, or public distribution is added.

Use the exact generated filename below; select fresh isolated install/data folders:

```powershell
./scripts/desktop-smoke.ps1 -Step Install -DataDirectory 'D:/projects/nothing-to-see-here/rr/target/desktop-smoke/package-data' -InstallDirectory 'D:/projects/nothing-to-see-here/rr/target/desktop-smoke/package-installed' -PackagePath './target/release/bundle/nsis/Local Stack Proof_0.1.0_x64-setup.exe'
./scripts/desktop-smoke.ps1 -Step Launch -DataDirectory 'D:/projects/nothing-to-see-here/rr/target/desktop-smoke/package-data' -InstallDirectory 'D:/projects/nothing-to-see-here/rr/target/desktop-smoke/package-installed'
```

The script is Windows-specific verification tooling; core/UI paths remain
host-agnostic. Substitute your absolute checkout path. It checks paths, refuses
an existing install directory, does not delete data, and only accepts this project's
local package. NSIS installation writes current-user shortcuts/uninstall registration
as normal. Close the app, then run Launch again against the same data directory.

Stop the project API/Vite and `docker compose -f infra/compose.yaml stop postgres`
before launch. Do not terminate unrelated processes or reset volumes. In the UI,
create/execute default seed 42, north, threshold 0 once normally and once with forced
rejection; verify all routes/diagnostics and note the exact IDs. After restarting,
retrieve both IDs, then run:

```powershell
./scripts/desktop-smoke.ps1 -Step Verify -DataDirectory 'D:/projects/nothing-to-see-here/rr/target/desktop-smoke/package-data' -CompletedId 'run-00000000000000000001' -RejectedId 'run-00000000000000000002'
docker compose -f infra/compose.yaml up -d --wait
```

Verify checks existing data read-only using the Rust `sqlite-proof` utility. This
supplements, not replaces, visual UI observations. It never creates a missing DB,
migrates, or writes runs. Read-only behavior is tested. Inspect listeners for the
installed executable and project server ports separately; do not disable host-wide
network/security settings. Machine-wide offline behavior/WebView background traffic
is not claimed from stopped project servers alone.

Installed builds keep `desktop.log` beside the run DB: startup and safe request
method/route-template/status/timing only, no request bodies, IDs, credentials,
filesystem paths, remote logging, or telemetry. Logging stops near 1 MiB without
truncating or deleting prior logs. SQLite/ordinary app data is outside installation
files and retained across app close/reinstall; uninstall data-removal choices are
not exercised by this smoke. Test data and installed files are kept for inspection.
Frontend capabilities are explicitly empty: no shell, filesystem, dialog, HTTP,
or updater plugin permission. The Rust host accesses only its selected local store.
