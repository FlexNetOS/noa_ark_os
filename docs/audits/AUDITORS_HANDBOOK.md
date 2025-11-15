# Auditors' Handbook

This handbook is generated automatically from the immutable relocation and documentation ledgers together with active SOP references. It reflects the state as of 2025-11-12 03:23 UTC.

## Ledger Overview

| Ledger | Entries | Last Event | Scope |
| --- | --- | --- | --- |
| Relocation | 1 | 2025-11-12 03:22 UTC | relocation_pipeline |
| Documentation | 1 | 2025-11-12 03:22 UTC | docs/audits/AUDITORS_HANDBOOK.md |

## Relocation Ledger Detail

| Timestamp | Actor | Target | Signature |
| --- | --- | --- | --- |
| 2025-11-12 03:22 UTC | tester | /tmp/target | `a832473419384033` |


## Documentation Ledger Detail

| Timestamp | Actor | Target | Signature |
| --- | --- | --- | --- |
| 2025-11-12 03:22 UTC | tester | docs/audits/AUDITORS_HANDBOOK.md | `3a3890027651103a` |

## SOP Alignment

The first steps from the operational SOP are included for quick reference:

````markdown
# Standard Operating Procedure: Development

## Purpose
Establish consistent development practices across the NOA ARK OS project.

## Scope
All development activities including coding, testing, and documentation.

## Procedure

````

## Verification Checklist

- Confirm ledger signatures chain correctly against the policy secret.
- Validate that handbook regeneration runs after each SOP execution.
- Cross-check relocation destinations with signed documentation targets.
