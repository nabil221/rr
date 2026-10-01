# M5-01: SQLite Persistence

**Status:** planned  
**Dependencies:** M2-04

## Objective

Provide self-contained desktop persistence through the same repository port used by Postgres.

## Scope

Implement SQLite migrations, repository mapping, per-user application-data location, and test-directory override. Exclude web/database sync.

## Verification

- Run common repository-contract tests where applicable.
- Restart against the same test database and retrieve previous run history.

## Acceptance criteria

- Desktop state survives restart.
- Tests can create isolated data without touching ordinary user data.
- No desktop operation requires Postgres or network access.

## Evidence

_Pending execution._
