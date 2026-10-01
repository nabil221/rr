# Issue Conventions

Each issue is a self-contained execution packet. The agent updates its `Status` and `Evidence` sections when finishing work.

## Required fields

- **Status:** `planned`, `in progress`, `blocked`, or `complete`.
- **Dependencies:** issues that must be complete first.
- **Objective:** the single outcome this issue owns.
- **Scope:** allowed work and explicit exclusions.
- **Implementation plan:** bounded steps, not a second roadmap.
- **Verification:** commands or manual checks that prove the result.
- **Acceptance criteria:** binary completion conditions.
- **Evidence:** filled only after execution with commands, results, and limitations.

An issue must not silently broaden its own scope. Create or amend an ADR for a material architectural decision.

## Detours

Work that interrupts the ordered roadmap goes in a standalone record under `docs/detours/` before implementation. Use `docs/detours/README.md` for the register, status meanings, and required fields. A detour has a precise return point and does not itself authorize starting the next roadmap issue.
