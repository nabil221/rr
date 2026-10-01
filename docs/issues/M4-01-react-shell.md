# M4-01: React Shell and Client Boundary

**Status:** complete  
**Dependencies:** M3-03, whichever desktop transport path (M2-02 or M2-03) is selected

## Objective

Harden and expand the M1-04 proof UI into the host-neutral React shell used by the persistent browser and desktop flows.

## Scope

Refactor the thin M1-04 screen into routes for Runs, New Run, Detail, and Diagnostics; add loading/empty/error states; configurable API endpoint; and a client interface. Exclude full forms and desktop packaging.

## Implementation plan

1. Introduce a typed `RunClient` interface with injectable screen dependencies,
   generated DTOs, detail/health methods, and safe host/storage diagnostics.
2. Keep Tauri detection and endpoint selection exclusively in the client adapter.
   Accept only credential-free loopback HTTP roots for optional `VITE_API_BASE_URL`;
   default to the existing same-origin proxy. Native ignores that override.
3. Use a small hash router for Runs, New Run, Detail, and Diagnostics so deep links
   and refresh work in bundled desktop assets without server-side route fallback.
   No routing dependency is necessary for these four fixed proof routes.
4. Extract shared loading/empty/error presentation and responsive navigation.
   Scaffold screens now; complete their workflows in M4-02/M4-03.
5. Add client boundary and static route-render checks without new dependencies;
   interactive component/browser test dependencies belong to M4-04.
6. Run aggregate validation, record proof, and commit before the next issue.

## Verification

- Type-check and render each route.
- Confirm screen components depend on client interface, not direct browser or Tauri APIs.

## Acceptance criteria

- UI is usable at normal desktop widths and resilient at narrow widths.
- No domain calculation is reimplemented in React.
- Host identity is not scattered through UI code.

## Evidence

- `npm run validate` passed: 13 client/shell tests, 29 Rust tests, generated
  contract, TypeScript, lint/format, and both host/web builds.
- `npm run test:e2e -- -- shell.spec.ts` passed against isolated local processes:
  all four routes at 1120px and 360px, no document overflow, navigation and refresh.
- `RunClient` injection keeps screen modules free of fetch, Tauri APIs, endpoint,
  and host detection. Adapter tests cover local-only overrides, native override
  isolation, encoded detail IDs, and allowlisted safe diagnostics.
- DET-002 bootstrapped the planned local test tools before issue verification;
  standalone screens remain deliberately scaffolded until M4-02/M4-03.
