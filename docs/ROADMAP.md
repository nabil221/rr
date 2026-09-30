# Local Stack Proof - Roadmap Index

This file is the ordered execution index. Each issue is standalone so an agent can take one bounded work packet, prove it, and leave a useful handoff.

Read before starting work:

- [Architecture charter](architecture.md)
- [Agent operating contract](agent-operating-contract.md)
- [Issue conventions](issues/README.md)
- [Proof matrix](proof-matrix.md)
- [Detour register](detours/README.md)

## Target proof

```text
browser / optional PWA -> local Axum API -> local Postgres
Tauri desktop          -> local Rust host -> local SQLite
```

Both modes create, execute, inspect, and persist a deterministic demo run. The stores do not synchronize.

## Ordered backlog

M4 is complete: routed shared UI, queued creation and persistent history, observable
execution/detail, safe diagnostics, and repeatable local browser/component tests.
M3's Postgres/contract proof and M2's native baseline remain verified. Desktop storage
is still in-memory until M5. The user authorized M5: SQLite, native integration,
and Windows package verification. M5-01/02 are complete; M5-03 is in progress. Stop before M6.

| Order | Issue                                                                                      | Depends on                     |
| ----: | ------------------------------------------------------------------------------------------ | ------------------------------ |
|     1 | [M0-01 Repository foundation](issues/M0-01-repository-foundation.md)                       | none                           |
|     2 | [M0-02 Local configuration and Postgres](issues/M0-02-local-configuration-postgres.md)     | M0-01                          |
|     3 | [M0-03 Documentation foundation](issues/M0-03-documentation-foundation.md)                 | M0-01                          |
|     4 | [M1-01 Domain model](issues/M1-01-domain-model.md)                                         | M0-01                          |
|     5 | [M1-02 Deterministic computation](issues/M1-02-deterministic-computation.md)               | M1-01                          |
|     6 | [M1-03 Application services](issues/M1-03-application-services.md)                         | M1-01, M1-02                   |
|     7 | [M1-04 In-memory web vertical slice](issues/M1-04-in-memory-web-vertical-slice.md)         | M1-03                          |
|     8 | [M2-01 Desktop transport decision](issues/M2-01-desktop-transport-decision.md)             | M1-04                          |
|     9 | [M2-02 Embedded desktop transport spike](issues/M2-02-embedded-desktop-spike.md)           | M2-01                          |
|    10 | [M2-03 Fallback transport](issues/M2-03-fallback-transport.md)                             | M2-02 fails only               |
|    11 | [M2-04 In-memory desktop vertical slice](issues/M2-04-in-memory-desktop-vertical-slice.md) | selected M2 path, M1-04        |
|    12 | [M3-01 Postgres persistence](issues/M3-01-postgres-persistence.md)                         | M1-03, M0-02                   |
|    13 | [M3-02 Axum API](issues/M3-02-axum-api.md)                                                 | M3-01, M1-04                   |
|    14 | [M3-03 API contract](issues/M3-03-api-contract.md)                                         | M3-02                          |
|    15 | [M4-01 React shell](issues/M4-01-react-shell.md)                                           | M3-03, selected M2 path        |
|    16 | [M4-02 Create and list UI](issues/M4-02-create-list-ui.md)                                 | M4-01                          |
|    17 | [M4-03 Execute, detail, diagnostics UI](issues/M4-03-execute-detail-diagnostics-ui.md)     | M4-02                          |
|    18 | [M4-04 Browser automation](issues/M4-04-browser-automation.md)                             | M4-03                          |
|    19 | [M5-01 SQLite persistence](issues/M5-01-sqlite-persistence.md)                             | M2-04                          |
|    20 | [M5-02 Desktop integration](issues/M5-02-desktop-integration.md)                           | M5-01, M4-01, selected M2 path |
|    21 | [M5-03 Windows package smoke test](issues/M5-03-windows-package-smoke-test.md)             | M5-02                          |
|    22 | [M6-01 Evidence and demo](issues/M6-01-evidence-demo.md)                                   | M4-04, M5-03                   |
|    23 | [M6-02 Optional PWA](issues/M6-02-optional-pwa.md)                                         | M6-01                          |
|    24 | [M6-03 Final acceptance](issues/M6-03-final-acceptance.md)                                 | M6-01; M6-02 if selected       |

## Execution rules

- Work in order; skip M2-03 unless M2-02 fails its criteria.
- PWA is optional and cannot delay browser or desktop proof.
- A completed issue must have passing evidence in its own document and in the proof matrix.
- Stop at the authorized milestone boundary and obtain confirmation before starting the next milestone. Current authorization covers M5 only; M6 is not authorized.
- Pause only for a hard-boundary change: remote services, synchronization, accounts, a technology-family replacement, paid infrastructure, real-file processing, or retaining a failed transport approach.
- A bounded detour must be recorded in the [detour register](detours/README.md) before implementation begins. A detour never advances the ordered issue backlog or overrides an explicit milestone/issue stop.
