# M3-03: API Contract

**Status:** complete  
**Dependencies:** M3-02

## Objective

Make the Rust HTTP contract and TypeScript client unable to silently drift.

## Scope

Generate/maintain OpenAPI and generated TypeScript types/client or an equally strict compile-time validation path. Exclude unrelated client features.

## Implementation plan

1. Derive TypeScript definitions directly from the Rust HTTP DTOs using `ts-rs`;
   keep the domain and storage mapping independent. This is the strict generated-types
   alternative to OpenAPI permitted by this issue, not a second schema maintained by hand.
2. Generate one checked-in TypeScript contract module through a small Rust binary.
   Its `--check` mode compares exact output and fails on drift; aggregate validation
   runs this before the browser typecheck. No implicit generation during ordinary tests.
3. Replace hand-written client interfaces with generated imports, including explicit
   string-union status/event types. Keep transport selection behind the existing client boundary.
4. Restrict HTTP seed values to JavaScript's safe unsigned integer range and map Rust
   `u64` to TypeScript `number` explicitly. Do not silently round client data.
5. Document success, controlled rejection, and invalid-request examples. Prove drift
   detection with a temporary incompatible DTO rename, restore it, and rerun checks.
6. No schema validator, OpenAPI server, SDK framework, route redesign, or new UI work.

## Verification

- Run contract generation/check.
- Make a temporary incompatible DTO change and confirm validation fails, then revert it.

## Acceptance criteria

- API examples represent success and validation failure.
- Browser client compiles from the same authoritative contract.
- Contract validation is included in aggregate local checks.

## Evidence

- `cargo run -p local-stack-proof-transport-http --bin api-contract -- --write`
  generated the committed module directly from all 11 Serde DTO/enum definitions,
  using `ts-rs` 12.0.1. No auto-export tests rewrite source files.
- A temporary `#[serde(rename = "runIdentifier")]` on `RunDto.id` caused
  `api-contract --check` to exit 1 with the expected drift error. Removed the
  temporary attribute; the same check then passed. The generated file was not
  rewritten to mask the incompatible experiment.
- `npm run validate` passed: explicit contract check before TypeScript compilation,
  formatting, Oxlint, Clippy, 29 Rust unit tests, 5 client tests, and web/Rust builds.
- The committed-output Rust test passes; safe seed boundary accepts 9007199254740991
  and rejects the next integer with a typed 400 error. Both hosts use the generated
  status/event unions through the shared frontend client.
- Three explicit live Postgres tests passed again; a final psql inspection found
  no leftover generated test schemas.
- [HTTP contract documentation](../api-contract.md) records routes, full create/error
  examples, clearly labeled success/rejection excerpts, generation workflow, and limitations.
- This guarantees DTO declaration alignment under validation, not runtime validation
  of arbitrary JSON or an OpenAPI/generated-route SDK. Route semantics remain
  covered by existing router/client tests. M4 UI work has not started.
