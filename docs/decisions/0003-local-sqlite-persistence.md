# ADR-003: Local SQLite Persistence

**Status:** accepted and adapter-verified in M5-01

## Decision

## Context and options

The desktop already reaches the shared Rust application through an embedded router.
It needs an independent file-backed store, not another process. Bundled SQLite with
the existing synchronous ports avoids a host-wide async-port rewrite or a system
SQLite prerequisite. Postgres remains exclusive to the browser API.

## Storage design

Use bundled rusqlite behind the same synchronous repository/ID ports as Postgres.
SQLite is compiled into the desktop binary: no separate database service or runtime
installation. Host requests continue using blocking workers. A mutex protects each
connection; immediate write transactions and a five-second busy timeout coordinate
independent connections. This is a local proof, not a throughput claim.

Store the same validated snapshots and ordered, append-only events as Postgres,
sharing serialization in a persistence-format crate. SQL remains adapter-specific.
Commit each snapshot/event transition atomically and reject stale prefixes/config.
Allocate IDs in a committed database sequence before insertion; gaps are allowed.
Read snapshot and events in one transaction and replay the lifecycle without
recomputing results. Unknown schema versions or corrupt state fail safely.

Tauri resolves its per-user application-local-data directory. An explicit absolute
`LOCAL_STACK_PROOF_DATA_DIR` overrides it for isolated walkthroughs; it never selects
Postgres or a remote store. The database is `runs.sqlite3` within that directory.
No automatic reset, sync, import, or retry/recovery of interrupted running jobs.

## Verification

Temporary-directory adapter tests, native SQLite protocol tests, native development
walkthrough, and installed Windows-package restart readback. M5-01 adapter tests and
full web regression suite pass; host/package evidence remains M5-02/03.

## Consequences and fallback

Independent stores do not synchronize. IDs are unique within each store, not across
hosts. A committed STARTED transition can survive a process interruption as running;
automatic recovery is outside scope. Connection failures do not silently discard
data or fall back to memory. Keep the database and diagnose the failed check before
proceeding; rollback must not reset ordinary user data.

## Sources

- [rusqlite transaction behavior](https://docs.rs/rusqlite/0.40.2/rusqlite/enum.TransactionBehavior.html)
- [Tauri application paths](https://docs.rs/tauri/2.12.0/tauri/path/struct.PathResolver.html)
- [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/)
