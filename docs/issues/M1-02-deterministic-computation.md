# M1-02: Deterministic Computation

**Status:** complete  
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

## Evidence

- `cargo test -p local-stack-proof-domain` passed: 7 tests, including a fixed seed `42` / region `north` fixture (`24` generated, `7` matched, total score `449`, average `449/7`) and stable category aggregates.
- The same suite verifies repeatability, threshold rejection, empty-region rejection, and forced rejection as `ValidationMessage` rather than a panic.
- `cargo clippy -p local-stack-proof-domain --all-targets -- -D warnings` passed.
- `calculate` is deterministic and depends only on its configuration; no clock, filesystem, database, network, or external crate is used.
