# DET-001: Oxlint and TypeScript 7 Evaluation

**Status:** complete  
**Roadmap position:** after M1-01; this detour does not start M1-02  
**Authorization:** user authorized evaluating/adopting Oxlint and explicitly asked that the detour process be documented first. This record was created after dependency work began; see chronology below.

## Why now

The scaffold currently uses TypeScript 5.9.3 because its ESLint integration (`typescript-eslint`) declared a peer range excluding TypeScript 7. The user asked whether Oxlint is a sensible alternative, then authorized trying it. This is a bounded tooling evaluation while the lint configuration is still small.

## Intended outcome

Determine whether Oxlint can replace the current ESLint setup, preserve the active lint checks, and allow a TypeScript 7 build in this repo. Keep the project local and leave M1-02 untouched.

## Scope

- Replace the web workspace's ESLint invocation/config/dependencies with Oxlint, preserving the configured correctness, React Hooks, and Fast Refresh checks.
- Upgrade the web workspace to the registry's current stable TypeScript 7 release and verify the existing build/test tooling.
- Preserve or consciously document behavior for the existing recommended TypeScript checks, React Hooks checks, and React Fast Refresh export check.
- Update package manifests/lockfile, scaffold/version evidence, roadmap/detour register, and proof matrix with actual results.

## Exclusions

- No M1-02 or later roadmap issue implementation.
- No adding type-aware linting unless the base Oxlint + TypeScript 7 checks expose a specific need and the scope is amended first.
- No changes to the app's runtime architecture, frameworks, API/database, formatting tool, or Rust toolchain.
- No claims of full ESLint/typescript-eslint rule parity; only verify checks currently configured in this repo.

## Boundary and approval

Changing from ESLint to Oxlint is a lint-tool family replacement and therefore a hard-boundary change under the operating contract. The user explicitly authorized this Oxlint evaluation. Authorization is limited to the scope above; TypeScript 7 is an associated compatibility test. Any unrelated architecture/tool-family expansion requires approval.

## Verification and acceptance

- Oxlint runs as the web workspace lint command, with configuration covering the prior project's active checks.
- The TypeScript 7 compiler passes web typecheck, web tests, and web build; any failure is recorded and TypeScript is restored if the failure is outside the approved acceptable outcome.
- Root validation passes (`npm run validate`) or each failure/limitation is recorded with a concrete cause.
- ESLint dependencies/configuration are removed only if rule behavior is represented and checks pass.
- The roadmap remains positioned after M1-01; no M1-02 work is started.

## Outcome

- Replaced ESLint, `typescript-eslint`, and their React/JS plugin dependencies with Oxlint 1.86.0. Added `apps/web/.oxlintrc.json`; the workspace lint script runs Oxlint against `src` and `vite.config.ts`.
- Upgraded TypeScript to 7.0.2. Its stricter side-effect import check initially rejected `import "./styles.css"`; adding `src/vite-env.d.ts` with Vite's client declarations resolved this without weakening compiler checks.
- The first aggregate validation attempt stopped at Prettier for the newly added detour register; formatting was fixed. The next attempt passed `npm run validate` (format, TS typecheck, Oxlint, Clippy, Vitest, Cargo tests, and production builds).
- No type-aware Oxlint mode was added: it was outside the need being tested. This proves the project's configured lint checks and TypeScript 7 build path, not complete ESLint rule parity.
- The full workspace validation includes Rust tests; M1-01 remains the last roadmap issue. No M1-02 work began.

## Stop condition and return point

The detour is closed after passing its acceptance criteria. Return to M1-02, but do not begin it until the user confirms, consistent with the existing milestone/issue stop instruction.

## Chronology and evidence

- The user asked for a detour process and Oxlint evaluation. Before creating this record, the agent queried registry versions and began changing the web dev dependencies. This was out of order; after the user pointed it out, implementation paused until the plan and process docs were added.
- Registry query (2026-09-28): `oxlint` latest `1.86.0`; `typescript` latest `7.0.2`; `oxlint-tsgolint` latest `7.0.2003`. The latter was not installed.
- Final verification: `npm run validate` — passed (exit code 0), including web typecheck/build, Oxlint, Vitest, Rust Clippy/tests/build, and formatting checks.
