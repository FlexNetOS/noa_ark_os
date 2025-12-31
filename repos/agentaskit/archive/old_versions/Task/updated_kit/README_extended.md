# NOA Deployment Kit v3.1 — Extended Metadata Edition

This directory extends the original **NOA Deployment Kit v3.1** by
adding support for the rich metadata contained in the
**All Inclusive Agent Directory v6+** CSV.  All original files remain
untouched; you can continue to use the v1 schema and normaliser.  The
new files provide a drop‑in upgrade path for consumers that need
additional context such as purpose descriptions, capability packs,
governance flags, dependencies and budgeting information.

Highlights:

* `CSV_SCHEMA_v2.md` – documents the optional metadata fields in
  addition to the mandatory v1 columns【928610657846884†L2-L8】.
* `normalize_csv_v2.py` – heals the CSV and keeps **all** columns,
  producing `All_Inclusive_Agent_Directory_v6_plus.normalized.full.csv`
  (81 columns, 862 rows).
* `capsule.schema.v2.json` – extended JSON schema for capsules.  It
  introduces `metadata`, `dependencies`, `governance` and
  `monitoring` objects to capture the new fields.
* `stack.manifest.v2.json` – generated from the full normalised CSV
  using the v2 schema.  This file preserves the CECCA→Board→Stack
  hierarchy and embeds the additional metadata for each capsule.
* `HOW-TO-USE_v3_1_extended.md` – step‑by‑step guide for using the
  extended tooling.

When upgrading existing workflows, be mindful of the **heal, don’t
break** principle: the v2 files add information but never remove or
rename the mandatory fields defined in the v1 schema.