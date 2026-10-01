# M4-04: Browser Automation

**Status:** planned
**Dependencies:** M4-03

## Objective

Automate the main browser proof paths against local processes.

## Scope

Add component tests plus browser automation for create, execute, inspect, rejection, and refresh persistence. Exclude remote test services.

## Verification

- Run automation against locally started API/UI/database.
- Confirm tests wait on UI/API state rather than arbitrary timeouts.

## Acceptance criteria

- Happy and rejected end-to-end paths pass reproducibly.
- Failure output identifies the failed stage clearly.


