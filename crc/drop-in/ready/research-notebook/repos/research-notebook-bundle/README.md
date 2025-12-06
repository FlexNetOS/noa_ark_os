# Research Notebook Bundle (Ready)

This bundle contains sanitized Jupyter notebooks and companion datasets that passed CRC intake validation and are ready for promotion.

## Contents

- `notebooks/development/01_core_os_development.ipynb` – core engineering workflow notebook with automation metadata cells.
- `notebooks/analysis/performance_analysis.ipynb` – metrics review notebook guarded by dataset existence checks.
- `datasets/workspace_metrics_daily.csv` – daily CRC metrics snapshot referenced by analysis notebooks.
- `evidence/ledger.json` – recorded sanitization metadata and test executions.
- `manifest.json` – machine-readable description for orchestrator automation.

## Usage

1. Create and activate a virtual environment.
2. Install dependencies from `notebooks/requirements.txt` at the repository root.
3. Launch Jupyter Lab and open the notebooks.
4. Update `evidence/ledger.json` with any additional verification performed prior to production promotion.
