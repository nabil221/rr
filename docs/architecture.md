# Architecture Charter

## Intent

The project proves a local, two-host architecture. It is not a product prototype.

```text
React + TypeScript UI
  |                     |
browser / PWA           Tauri desktop
  |                     |
local Axum API          selected local desktop transport
  |                     |
Postgres                SQLite
  \                     /
   shared Rust domain and application crates
```

## Dependency direction

`domain` has no host, transport, persistence, clock, filesystem, or network dependency.

`application` depends on `domain` and defines ports. It must not depend on Axum, Tauri, SQL drivers, React, or concrete storage.

`persistence` implements application ports. `transport-http` maps HTTP DTOs to application calls. `apps/api` and `apps/desktop` compose concrete adapters.

## Local deployment modes

| Mode          | UI                          | Host                             | Storage        | Network requirement |
| ------------- | --------------------------- | -------------------------------- | -------------- | ------------------- |
| Browser       | React/Vite                  | local Axum process               | local Postgres | localhost only      |
| PWA, optional | same React build            | local Axum process               | local Postgres | localhost only      |
| Desktop       | same React production build | Tauri + selected local transport | SQLite         | none                |

The browser/PWA database and desktop database are deliberately independent. No synchronization is designed, implied, or tested.

The first browser slice intentionally uses an in-memory repository. That is a temporary proof stage, not a third deployment mode: it establishes the frontend-to-application interaction before the Postgres adapter is introduced.

## Non-goals

- Hosting, cloud accounts, SaaS deployment, telemetry, third-party login, or remote APIs.
- Excel/CSV parsing, spreadsheet compatibility, real workflow automation, and data-product features.
- Generic job queue, workflow DSL, plugin system, mobile application, multiple users, or desktop/web data synchronization.

## Scope test

Work belongs in this project only if it proves UI reuse, Rust application reuse, a local host/persistence/packaging boundary, or meaningful diagnosability of a failure.
