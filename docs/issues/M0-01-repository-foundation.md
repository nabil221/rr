# M0-01: Repository Foundation

**Status:** complete  
**Dependencies:** none

## Objective

Create a reproducible Rust and TypeScript workspace with one command for local quality checks.

## Scope

Create Cargo/npm workspaces, `apps/web`, `apps/api`, `apps/desktop`, crate placeholders, format/lint/test scripts, and a root README. Do not add business features or runtime services.

## Implementation plan

1. Create the workspace directory structure from the architecture charter.
2. Add minimal compilable Rust crates and React/Vite application.
3. Add package scripts for format, lint, test, and aggregate validation.
4. Document required local tool versions and bootstrap commands.

## Verification

- Run Rust format, lint, and test checks.
- Run TypeScript format, lint, type-check, and test checks.
- Run the aggregate validation command.

## Acceptance criteria

- All empty workspace members build without warnings.
- One documented command runs the full local validation set.
- No global JavaScript tool beyond the documented runtime/package manager is needed.

## Evidence

- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed.
- `npm run validate` passed for the completed foundation; the current toolchain update is evidenced separately in DET-001.
- Frontend Vitest: 1 test passed.
- Frontend production build and all Rust workspace binaries built successfully.
- Current version audit after DET-001: Node 24.21.0, npm 12.1.0, Vite 8.3.1, React 19.3.0, Vitest 5.0.2, Oxlint 1.86.0, TypeScript 7.0.2, and Rust 1.98.1.
- TypeScript 7 exposed its stricter side-effect import check for the CSS import; `src/vite-env.d.ts` now loads Vite's asset declarations. The updated toolchain passes the full validation command.
