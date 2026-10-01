# M2-04: In-Memory Desktop Vertical Slice

**Status:** complete  
**Dependencies:** selected M2-02 or M2-03 transport path, M1-04

## Objective

Prove the complete frontend-to-Rust flow inside the desktop application before introducing SQLite.

## Scope

Run the M1-04 interaction through the selected desktop transport and the same in-memory application state: create, execute, display result, and display controlled rejection. Exclude SQLite, Postgres, sync, installer work, and UI redesign.

## Implementation plan

1. Reuse the M1-04 thin React flow and in-memory application fixture.
2. Select the desktop client transport established by M2-02 or M2-03.
3. Exercise success and rejection through the packaged/development Tauri host as appropriate for the spike.
4. Record that state is intentionally lost on restart.

## Verification

- Stop the browser API and Postgres.
- Launch the desktop development target.
- Create and execute a successful run from the desktop UI.
- Trigger controlled rejection and observe the desktop-rendered error.

## Acceptance criteria

- The complete desktop frontend-to-backend loop works with no database.
- The same Rust application service handles browser and desktop execution.
- The desktop UI does not contain duplicated domain or calculation logic.
- Persistence is explicitly marked as the next concern, not treated as a transport requirement.

## Evidence

- M2-02 accepted the embedded-router transport. Implementation reuses the existing React screen and selects host URLs only inside `api.ts`.
- `npm run validate` passed: formatting, TypeScript, Oxlint, warning-free Rust Clippy, 5 frontend client tests, 20 Rust tests, and web/Rust workspace builds. The React copy now describes both hosts; no screen-specific Tauri business branches were added.
- `npm run dev:desktop` opened the normal React entry point and logged `GET /api/runs -> 200` through the in-process router. This proves screen startup/client resolution, not button interactions or visual layout.
- `npm run build:desktop` passed. The final production executable opened the shared React entry point and logged `GET /api/runs -> 200`; see local generated log `target/m2-evidence/desktop-production-20260930-113327.stderr.log`. An elevated `netstat -ano -p tcp` check confirmed no listeners on API/Vite/Postgres ports 3000/5173/5174/54329 and no listener owned by desktop process 24960. The production app was left open for inspection.
- Native acceptance resumed after the user switched this chat from CLI to the desktop app. Computer Use then connected successfully; the former helper-connection blocker is resolved.
- Production native React walkthrough passed on 2026-09-30: seed 42, North, threshold 0 created/executed `run-000001` with completed status, 24 generated records, 7 matched records, average 64.14, and category breakdown. Enabling controlled rejection created/executed `run-000002`, rendered the validation message, and exposed CREATED/STARTED/REJECTED lifecycle events. Both runs appeared in session history.
- Ctrl+R reloaded the frontend: the result selection reset while both history rows remained. Restarting the verified desktop process cleared history to 0 runs. These were intentionally temporary demo runs; no persistent files or database records were deleted.
- Native screenshots and accessibility state confirmed the result/rejection screens. An elevated port check during the walkthrough confirmed no API, Vite, or Postgres listener on ports 3000/5173/5174/54329. Development transport/startup and production full UI acceptance are both covered; installer/distribution and persistence remain later work.
- M2 is complete. The restarted production app is left open with empty history for inspection. M3 remains unauthorized.
