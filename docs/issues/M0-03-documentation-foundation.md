# M0-03: Documentation Foundation

**Status:** complete  
**Dependencies:** M0-01

## Objective

Establish living architecture, decision, and proof documents before implementation produces undocumented assumptions.

## Scope

Complete the architecture charter, decision-record convention, proof matrix, and root links. Do not create speculative design documents.

## Implementation plan

1. Confirm deployment modes and dependency direction match the charter.
2. Ensure non-goals and local-only constraints are explicit.
3. Create ADR and evidence templates if missing.
4. Link the documents from the roadmap index.

## Verification

- Review documents against the target proof.
- Confirm all architecture-changing work has a designated ADR location.

## Acceptance criteria

- A new agent can identify shared code, host-specific code, and boundaries without chat history.
- The documents forbid cloud, sync, accounts, Excel processing, and a generic workflow engine.

## Evidence

- Architecture charter, agent operating contract, proof matrix, decision-record convention, and standalone issue packets are present.
- Roadmap links and issue-file integrity were checked after the in-memory web/desktop sequencing refinement.
- Formatting checks pass for the documentation set.
