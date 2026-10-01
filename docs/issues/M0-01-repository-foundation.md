# M0-01: Repository Foundation

**Status:** planned
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


