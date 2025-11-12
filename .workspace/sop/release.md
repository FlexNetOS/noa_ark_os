# Release Management SOP

## Purpose
Provide a repeatable process to take changes from staging to production while capturing documentation artifacts.

## Triggers
- Merge into `main` branch
- Manual release request via operations board

## Required Inputs
- Latest pipeline diff summary
- Approval records
- Deployment checklist

## Steps
1. Validate documentation agent report for the release candidate.
2. Confirm CI/CD doc refresh job status is `success`.
3. Review runbook verification matrix and ensure all automated checks are green.
4. Obtain approvals from release manager and documentation lead.
5. Execute production deployment using the preferred strategy.
6. Monitor verification dashboards for 30 minutes.
7. Archive documentation snapshot to `docs/wiki/releases/<version>.md`.

## Outputs
- Signed release approval stored in `.workspace/sop/approvals/`
- Updated `docs/runbook/release.md`
- Post-release retrospective entry in `docs/wiki/changelog.md`
