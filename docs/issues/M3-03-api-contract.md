# M3-03: API Contract

**Status:** planned
**Dependencies:** M3-02

## Objective

Make the Rust HTTP contract and TypeScript client unable to silently drift.

## Scope

Generate/maintain OpenAPI and generated TypeScript types/client or an equally strict compile-time validation path. Exclude unrelated client features.

## Verification

- Run contract generation/check.
- Make a temporary incompatible DTO change and confirm validation fails, then revert it.

## Acceptance criteria

- API examples represent success and validation failure.
- Browser client compiles from the same authoritative contract.
- Contract validation is included in aggregate local checks.


