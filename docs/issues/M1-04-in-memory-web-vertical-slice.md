# M1-04: In-Memory Web Vertical Slice

**Status:** complete  
**Dependencies:** M1-03

## Objective

Prove the first complete frontend-to-backend interaction before introducing any database, desktop bridge, or persistence complexity.

## Scope

Build a deliberately thin local web slice: React form and result view, Axum routes, application services, and an in-memory repository. The slice must create a demo run, execute it, and render the result or controlled rejection. It may use a small hand-written transport client temporarily. Exclude Postgres, SQLite, PWA, authentication, and UI polish.

## Implementation plan

1. Replace the API placeholder with local Axum routes backed by in-memory application state.
2. Add the smallest React screen capable of entering configuration, submitting a run, executing it, and rendering a result/error.
3. Keep application services and DTO boundaries visible; do not move computation into handlers or React.
4. Add a local start command that runs API and UI together, or document both commands clearly.

## Verification

- Start the local API and Vite app.
- Create and execute one successful run from the UI.
- Trigger controlled rejection and observe a useful error.
- Restart the API process, confirm history is empty, and document that this in-memory history is temporary. A browser refresh alone does not restart the API and should not erase its state.

## Acceptance criteria

- The complete browser interaction loop works before any database exists.
- Domain/application service tests remain green.
- Replacing the in-memory repository later does not require changing computation or screen contracts.
- The UI clearly labels this as a temporary in-memory proof.

## Evidence

- `npm run validate` passed: formatting, TypeScript, Oxlint, Vitest, all Rust tests and Clippy, plus production builds.
- The browser page is served by Vite on `127.0.0.1:5173`; the Vite `/api` proxy forwards to Axum on `127.0.0.1:3000`. `GET /api/health` through Vite returned `{ "status": "ok", "storage": "in-memory" }`.
- Exercised the same same-origin API endpoints used by the React client through Vite: create returned `queued`; execute returned `completed` with 24 generated records, 7 matches, deterministic grouped metrics, and created/started/completed events; list returned the run.
- Exercised the controlled validation path through Vite: execute returned `rejected` with one user-readable validation message.
- Restarted the API and confirmed `GET /api/runs` returned an empty list. Browser refresh is not expected to clear history; API process restart does.
- The production web bundle built successfully. Chrome UI verification completed on 2026-09-30: clicked Create and execute run with seed 42, North, and threshold 0; `run-000019` displayed completed, 24 generated records, 7 matches, average 64.14, category breakdown, and created/started/completed lifecycle events. Inspected the rendered result in a browser screenshot.
- Checked Force a controlled validation rejection and submitted from the UI; `run-000020` displayed rejected, a readable validation message, and created/started/rejected lifecycle events.
- Reloaded the page: all 20 session runs remained visible, including both new runs. Selected `run-000019` from history and confirmed its result rendered again. No warnings or errors were captured in the Chrome tab console during the walkthrough. Existing session history was preserved.

## Run locally

In two terminals at the repository root, run `npm run dev:api` and `npm run dev:web`, then open <http://127.0.0.1:5173>.
