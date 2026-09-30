# Local HTTP contract

Rust's Serde DTOs in `crates/transport-http` are authoritative for JSON bodies.
`ts-rs` generates `apps/web/src/generated/api-contract.ts`; the same client uses
these types in both browser and desktop. No independent TypeScript DTOs are maintained.

```text
npm run contract:generate
npm run contract:check
npm run validate
```

Generation is explicit. Check compares exact declarations (normalizing CRLF only),
and fails rather than rewriting the file. Aggregate validation checks the contract
before TypeScript compilation; Rust unit tests also check the committed output.
The generated module is intentionally excluded from Prettier so formatting cannot
hide or manufacture generator drift. Review DTO, generated module, and client changes together.

## Routes

| Method | Route                    | Successful JSON body |
| ------ | ------------------------ | -------------------- |
| GET    | `/api/health`            | `HealthDto`          |
| POST   | `/api/runs`              | `RunDto` (201)       |
| GET    | `/api/runs`              | `RunDto[]`           |
| GET    | `/api/runs/{id}`         | `RunDto`             |
| POST   | `/api/runs/{id}/execute` | `RunDto`             |

Create accepts `CreateRunRequest`. Seed is an unsigned integer from 0 through
9007199254740991, mapped explicitly to TypeScript `number`. Status and event kind
are generated string unions. Nullable fields are required properties with `null`,
not absent/optional properties. Property names follow Serde's camelCase mapping.

The typed client is not a runtime JSON validator or a generated route SDK. The
contract check proves DTO shape alignment, and router/client tests cover routes
and behavior. Remote/untrusted-server interoperability and OpenAPI tooling are not claimed.

## Examples

Create request (`POST /api/runs`, `Content-Type: application/json`):

```json
{
  "seed": 42,
  "region": "north",
  "threshold": 0,
  "forceValidationFailure": false
}
```

Creation returns 201 with a queued `RunDto`. Execute its returned ID with
`POST /api/runs/{id}/execute`. The successful response is 200; relevant excerpt:

```json
{
  "status": "completed",
  "result": {
    "totalRecords": 24,
    "matchedRecords": 7,
    "totalScore": 449,
    "averageScore": 64.14285714285714
  },
  "validationMessages": [],
  "events": [
    { "kind": "created", "message": "Run created" },
    { "kind": "started", "message": "Run started" },
    { "kind": "completed", "message": "Run completed" }
  ]
}
```

The full response also includes ID, configuration, and result groups. Creating
with `forceValidationFailure: true` and executing is a controlled business
rejection, **not** an HTTP transport error. Its 200 response excerpt is:

```json
{
  "status": "rejected",
  "result": null,
  "validationMessages": [
    {
      "code": "forced-rejection",
      "message": "Run rejected by the requested validation demonstration",
      "field": "force_validation_failure"
    }
  ]
}
```

The field identifier above is the existing domain validation identifier, not a JSON
property rename. A create request with an empty region returns 400, full error body:

```json
{
  "code": "invalid-request",
  "message": "region cannot be empty",
  "field": "region"
}
```

Malformed JSON returns 400 `invalid-json`; an oversized JSON body returns 413
`request-too-large`; missing runs/routes return 404; repeated execution or stale
writes return 409. Storage failures return a safe 500 without driver details.
Responses include server-owned `x-request-id` headers for log correlation.
Web accepts only explicit origin `http://127.0.0.1:5173`; the native protocol has
its separately proven exact-origin policy. Requests are capped at 64 KiB.
