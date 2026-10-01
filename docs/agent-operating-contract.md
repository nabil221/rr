# Agent Operating Contract

## Autonomous execution rule

The agent works the next unblocked issue in `docs/ROADMAP.md`, implements the smallest complete slice, runs the specified checks, then records evidence before moving forward. Normal implementation decisions are delegated to the agent.

The user's active execution boundary takes precedence: complete only the authorized milestone, then stop and request confirmation before starting the next milestone. M5 is complete; M6 is not authorized. A blocked acceptance check does not authorize skipping ahead.

## Per-issue procedure

1. Read the issue, its dependencies, relevant architecture rules, and active ADRs.
2. Inspect the current workspace and preserve unrelated user changes.
3. Implement only the issue's scope; avoid speculative abstractions.
4. Add or update the smallest automated test that proves the issue.
5. Run targeted checks, then the broadest practical local validation command.
6. Record test command, outcome, and any limitation in the proof matrix.
7. If a decision was made, add or update an ADR.

## Detours

A detour is a short, separately bounded task that interrupts the ordered issue backlog to address a blocker, correct an outdated assumption, or validate a material stack/tooling choice. It is not a way to silently pull later issue work forward.

Before changing code or dependencies for a detour:

1. Add a standalone record under `docs/detours/` and link it from the detour register. State the reason, expected outcome, scope and exclusions, risk/boundary, verification, stop condition, and exact return point in the issue backlog.
2. Get explicit user approval when the detour changes a technology family, recorded architecture decision, or another hard boundary. Approval of a particular detour authorizes only its written scope.
3. Keep the record `proposed` until the plan is written and authorized; mark it `in progress` before implementation, and record actual outcomes/deviations when closing it.
4. Run only the approved detour scope. If evidence calls for expanding it, pause and update the record/request approval before expansion.
5. Close the detour with evidence, update affected issue/docs/decision records, and return to its stated backlog point. Do not start that next issue unless separately authorized.

For a genuinely trivial, non-mutating investigation, a short note in the existing issue can suffice; dependency changes, config changes, and technology substitutions require a standalone detour record.

## Default technical choices

- Favor simple, typed, conventional interfaces over future-proof frameworks.
- Use polling for run status; do not add streaming unless a roadmap acceptance criterion fails without it.
- Use local-only configuration and project-scoped data. Never add a remote endpoint or credential silently.
- Keep React components host-agnostic; transport selection belongs behind a client boundary.
- Keep database-specific behavior behind application ports.

## Failure handling

- A test failure is work to resolve, not a reason to skip ahead.
- If a dependency is missing, install only the documented, project-scoped dependency after requesting any required system permission.
- If an experiment fails, collect the smallest reproducible evidence, record it, and take the roadmap's stated fallback.
- Do not retry the same failed approach more than twice without changing the hypothesis.

## Escalate only for boundary changes

Ask the user before adding remote infrastructure, accounts/authentication, data synchronization, a new technology family, paid services, processing real user files, or changing a recorded architecture decision. Otherwise, proceed.

## Definition of issue completion

An issue is complete only when the code works, relevant checks pass, docs reflect material decisions, and the proof matrix has evidence. A partial implementation is recorded as incomplete, not silently treated as done.
