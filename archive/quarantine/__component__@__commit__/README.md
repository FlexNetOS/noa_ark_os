# Quarantine Bundle Overview

> Replace the placeholder values in this document when quarantining code.

## Component Snapshot
- **Component Name:** `<component>`
- **Quarantined Commit:** `<commit-sha>`
- **Original Location:** `<path/in/repo>`
- **Owner / Steward:** `<team-or-individual>`
- **Point of Contact:** `<email-or-handle>`

## Reason for Quarantine
Describe why this component was quarantined, what risks were mitigated, and the expected remediation plan.

## Reintegration Gates
Document the conditions that must be satisfied before this component can be reintroduced:
1. Tests to restore or introduce
2. Architecture reviews or approvals required
3. Any new telemetry / observability hooks that must exist

## Hash Ledger
Use this table to track hashes for every file captured in this bundle.

| File | SHA-256 | Notes |
| ---- | ------- | ----- |
| `<relative/path>` | `<sha256>` | `<context>` |

## Reintegration Notes
- Outline the expected timeline for reintegration.
- Reference related issues, RFCs, or task IDs.
- Capture any downstream components waiting on this bundle.

## Rollback Instructions
List the exact `make rollback` command and ledger entry identifiers required to restore this bundle.
