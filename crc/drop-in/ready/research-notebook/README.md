# CRC Research Notebook Ready Bundle

This directory houses sanitized Jupyter notebook bundles that have completed intake processing and are cleared for CRC promotion.

## Structure

```
research-notebook/
├── manifest.json                   # Machine-readable manifest for the entire bundle family
├── README.md                       # Human operator guide
├── repos/                          # Ready bundles sourced from active repositories
├── forks/                          # Ready bundles sourced from forks (reserved)
├── mirrors/                        # Ready bundles sourced from mirrors (reserved)
└── stale/                          # Ready bundles sourced from legacy drops (reserved)
```

Only the `repos/` channel currently contains a ready bundle (`research-notebook-bundle`). Other channels include README placeholders and will be populated as additional notebook sources are certified.

## Promotion Workflow Summary

1. Validate the bundle metadata via `manifest.json`.
2. Follow the installation steps described in each bundle manifest.
3. Execute the CRC smoke tests noted in the evidence ledger to confirm ingestion.
4. Record additional evidence in `evidence/ledger.json` if new validation steps run prior to promotion.
