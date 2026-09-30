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

M5-01 adapter checks pass. M5-02 native walkthrough and M5-03 installed-package
smoke are in progress; no successful installation is claimed yet.
