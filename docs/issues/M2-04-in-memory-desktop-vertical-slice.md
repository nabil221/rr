# M2-04: In-Memory Desktop Vertical Slice

**Status:** planned
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


