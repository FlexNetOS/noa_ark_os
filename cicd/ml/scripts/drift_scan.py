#!/usr/bin/env python3
"""Offline drift detection for ML lifecycle artifacts."""

import argparse
import json
import pathlib
from typing import Dict, Any, List


def load_jsonl(path: pathlib.Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    data: List[Dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            try:
                data.append(json.loads(line))
            except json.JSONDecodeError as exc:
                raise SystemExit(f"Invalid JSON in {path}: {exc}")
    return data


def aggregate_by_lifecycle(entries: List[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    aggregated: Dict[str, List[Dict[str, Any]]] = {}
    for entry in entries:
        artifact = entry.get("artifact", {})
        lifecycle = artifact.get("lifecycle_id") or entry.get("lifecycle_id")
        if not lifecycle:
            continue
        aggregated.setdefault(lifecycle, []).append(entry)
    return aggregated


def detect_drift(artifact_entries: List[Dict[str, Any]], ledger_entries: List[Dict[str, Any]]):
    registry = aggregate_by_lifecycle(artifact_entries)
    ledger = aggregate_by_lifecycle(ledger_entries)

    report: List[str] = []
    for lifecycle_id, versions in registry.items():
        versions_sorted = sorted(versions, key=lambda item: item.get("timestamp", ""))
        baseline = versions_sorted[0]
        latest = versions_sorted[-1]
        baseline_metrics = baseline.get("metrics", {})
        latest_metrics = latest.get("metrics", {})
        drift_flags = []
        for metric, baseline_value in baseline_metrics.items():
            latest_value = latest_metrics.get(metric)
            if latest_value is None:
                continue
            try:
                baseline_float = float(baseline_value)
                latest_float = float(latest_value)
            except (TypeError, ValueError):
                continue
            if baseline_float == 0 and latest_float == 0:
                continue
            change = (latest_float - baseline_float) / (baseline_float or 1.0)
            if abs(change) > 0.05:
                drift_flags.append(f"{metric}: {baseline_float:.4f} -> {latest_float:.4f} ({change:+.2%})")
        ledger_versions = ledger.get(lifecycle_id, [])
        status = "clean" if not drift_flags else "drift"
        report.append(
            json.dumps(
                {
                    "lifecycle_id": lifecycle_id,
                    "status": status,
                    "issues": drift_flags,
                    "observations": len(ledger_versions),
                }
            )
        )
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description="Detect drift across ML artifacts")
    parser.add_argument("--artifact-log", type=pathlib.Path, required=True)
    parser.add_argument("--evidence-ledger", type=pathlib.Path, required=True)
    parser.add_argument(
        "--output",
        type=pathlib.Path,
        default=pathlib.Path("cicd/ml/reports/drift_report.jsonl"),
    )
    args = parser.parse_args()

    artifact_entries = load_jsonl(args.artifact_log)
    ledger_entries = load_jsonl(args.evidence_ledger)
    report_lines = detect_drift(artifact_entries, ledger_entries)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8") as handle:
        handle.write("\n".join(report_lines))

    print(f"Drift report written to {args.output}")


if __name__ == "__main__":
    main()
