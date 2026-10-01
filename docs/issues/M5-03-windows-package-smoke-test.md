# M5-03: Windows Package Smoke Test

**Status:** planned  
**Dependencies:** M5-02

## Objective

Produce and validate the actual local Windows package, not only desktop development mode.

## Scope

Configure package metadata/icons/logs and create a repeatable local install/launch/create/execute/restart smoke script. Exclude public distribution and code signing unless packaging requires a local prerequisite.

## Verification

- Build installer/package.
- Stop browser API and Postgres.
- Install and launch packaged application.
- Create, execute, restart, and retrieve a run.

## Acceptance criteria

- Packaged desktop proof passes with no browser-server process.
- Installer prerequisites and data-retention behavior are documented.
- Permissions are limited to what the proof actually needs.

## Evidence

_Pending execution._
