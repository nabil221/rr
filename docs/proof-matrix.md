# Proof Matrix

Update this document as issues are completed. A claim is not proven until it has a repeatable local check or a documented demo observation.

| Claim                               | In-memory web               | In-memory desktop   | Browser/Postgres | Persistent desktop | Evidence / command                                    | Status       |
| ----------------------------------- | --------------------------- | ------------------- | ---------------- | ------------------ | ----------------------------------------------------- | ------------ |
| Workspace quality gates             | proven                      | proven              | proven           | proven             | `npm run validate`; DET-001                           | proven       |
| Shared Rust domain/application code | proven                      | proven              | pending          | pending            | M2-02 shared router/service; Rust tests               | partial      |
| Frontend-to-backend interaction     | proven in Chrome UI         | proven in native UI | pending          | pending            | M1-04 Chrome; M2-04 production Tauri UI walkthrough   | partial      |
| Create, execute, and read run       | proven                      | proven              | pending          | pending            | M1-04 Chrome; M2-04 native results/history/reload     | partial      |
| Controlled validation failure       | proven                      | proven              | pending          | pending            | M1-04 and M2-04 forced rejection and lifecycle events | partial      |
| Persistence after restart           | n/a (in-memory)             | n/a (in-memory)     | pending          | pending            | M1-04 API and M2-04 desktop restart clear history     | pending      |
| HTTP contract/type alignment        | pending (hand-written DTOs) | n/a                 | pending          | n/a                | M1-04; formal shared contract remains future work     | pending      |
| Desktop works without API/Postgres  | n/a                         | proven              | n/a              | pending            | M2-02 spike and M2-04 native UI; API/Vite/DB stopped  | partial      |
| Windows packaged-build smoke path   | n/a                         | n/a                 | n/a              | pending            |                                                       | pending      |
| PWA installability, optional        | n/a                         | n/a                 | pending          | n/a                |                                                       | not selected |

## M3 adapter evidence

M3-01 is complete: three explicit live Postgres integration tests pass for
application-service create/execute/list/detail, reconnect readback, persistent IDs,
immutable event history, competing-write rejection, and atomic rollback.
`npm run validate` passes; ordinary tests skip the three explicit database tests.
No generated test schemas remained after execution. Browser/HTTP integration and
server-restart persistence remain pending M3-02; desktop SQLite remains pending M5.

## Explicitly not proven

- Deployment, remote/networked access, accounts, synchronization, mobile support, real file ingestion, real Excel/workflow functionality, job durability beyond this local process model, and generic workflow authoring.
