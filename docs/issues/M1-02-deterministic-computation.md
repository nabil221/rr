# M1-02: Deterministic Computation

**Status:** planned
**Dependencies:** M1-01

## Objective

Build a small, deterministic calculation that can prove execution and validation paths without becoming product functionality.

## Scope

Use seed, region, threshold, and forced-failure inputs to generate synthetic records, filter/group them, calculate summary metrics, and emit result or validation message. Exclude files, real datasets, and external libraries unless needed for the small proof.

## Implementation plan

1. Generate fixed records from a supplied seed.
2. Filter by region and derive one score.
3. Aggregate by a compact category field.
4. Apply threshold validation and an intentional rejection switch.
5. Document the algorithm in the domain crate or test fixtures.

## Verification

- Assert fixed results for known configurations.
- Assert deterministic repeated execution.
- Assert controlled rejection is a validation result, not a panic.

## Acceptance criteria

- Calculation has no clock, filesystem, database, or network dependency.
- Happy and rejected fixtures are easy to read and stable.


