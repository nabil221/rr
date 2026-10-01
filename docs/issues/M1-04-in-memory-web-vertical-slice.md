# M1-04: In-Memory Web Vertical Slice

**Status:** planned
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

## Run locally

In two terminals at the repository root, run `npm run dev:api` and `npm run dev:web`, then open <http://127.0.0.1:5173>.

