# Detour Register

Detours are deliberately small interruptions to the ordered roadmap. Each gets a standalone plan before implementation, a bounded approval, evidence, and an explicit return point. They do not change issue order or waive milestone/issue confirmation gates.

## Status meanings

- `proposed`: record exists, but work is not approved or started.
- `in progress`: the written scope is authorized and being executed.
- `complete`: acceptance criteria were met; evidence and backlog return point are recorded.
- `abandoned`: stopped without completing; reason and resulting workspace state are recorded.

## Required record fields

- Why the detour is needed now and the evidence prompting it.
- Intended outcome and concrete scope, including explicit exclusions.
- Boundary/approval: whether it changes a technology family, architecture decision, or other hard boundary, and what the user's approval covers.
- Verification and binary acceptance criteria.
- Stop condition and return point (specific roadmap issue and whether separate confirmation is required before starting it).
- Status, dates, outcomes, deviations, and workspace state if stopped.

## Register

| ID                                       | Detour                           | Status   | Return point                     |
| ---------------------------------------- | -------------------------------- | -------- | -------------------------------- |
| [DET-001](DET-001-oxlint-typescript7.md) | Evaluate Oxlint and TypeScript 7 | complete | M1-02, pending user confirmation |
