# M6-03: Final Acceptance and Cleanup

**Status:** planned
**Dependencies:** M6-01; M6-02 if selected

## Objective

Finish with an internally consistent, locally reproducible proof rather than an accumulation of working experiments.

## Scope

Run all local checks, remove dead spike code, audit dependencies, refresh architecture/ADRs/proof matrix, and record limitations. Exclude new features.

## Verification

- Run aggregate validation, browser automation, persistence tests, contract check, and packaged desktop smoke path.
- Verify domain/application crates build without hosts; web host builds without desktop host.

## Acceptance criteria

- Every selected roadmap issue is complete with evidence.
- No known failures are hidden or waived without an explicit limitation.
- Another developer can reproduce browser and desktop proof without external services.


