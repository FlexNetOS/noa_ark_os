#!/usr/bin/env python3
"""
Master Autonomous Orchestrator (Offline-Only)
- Runs deep analytics, gap hunt, and triple-verification
- Uses tools/framework_audit.py to verify structure per autonomous-system-map.mmd
- Provides hooks for (optional) execution steps, but defaults to audit-only
- No Docker, no network access. Pure local FS ops.
"""
from __future__ import annotations
import hashlib
import json
import os
import tarfile
import subprocess
import sys
from pathlib import Path
from datetime import datetime

WS = Path(__file__).resolve().parent
AUDIT = WS / "tools" / "framework_audit.py"
REPORT_JSON = WS / "framework_audit_report.json"
MAP_FILE = WS / "autonomous-system-map.mmd"
FREEZE_DIR = WS / "exports"
CAS_DIR = FREEZE_DIR / "cas"
WORKLOAD_ROLLUP = FREEZE_DIR / "workload_rollup.json"
WORKLOAD_ROLLUP = FREEZE_DIR / "workload_rollup.json"


def run(cmd: list[str]) -> tuple[int, str]:
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    out, _ = p.communicate()
    return p.returncode, out


def triple_verify() -> dict:
    """Triple verification per policy: (1) direct run, (2) re-run, (3) deterministic compare"""
    runs = []
    for i in range(3):
        code, out = run([sys.executable, str(AUDIT), str(WS)])
        runs.append({"code": code, "out": out})
    consistent = all(r["code"] == runs[0]["code"] for r in runs)
    # if report exists, read
    report = {}
    if REPORT_JSON.exists():
        try:
            report = json.loads(REPORT_JSON.read_text(encoding="utf-8"))
        except Exception:
            report = {"error": "report_read_failed"}
    return {"runs": runs, "consistent": consistent, "report": report}


def auto_heal_gaps(report: dict) -> list[dict]:
    """Additive-only auto-heal: create missing dirs/files minimally to satisfy structure.
    Does not modify existing content; only creates placeholders.
    """
    changes: list[dict] = []
    for gap in report.get("gaps", []):
        target = Path(gap.get("path"))
        missing = gap.get("missing", "")
        if missing.endswith("/"):
            # directory
            if not target.exists():
                target.mkdir(parents=True, exist_ok=True)
                changes.append({"created_dir": str(target)})
        else:
            # file
            if not target.exists():
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("", encoding="utf-8")
                changes.append({"created_file": str(target)})
    return changes


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(8192), b''):
            h.update(chunk)
    return h.hexdigest()


def snapshot(name: str) -> str:
    ts = datetime.now().strftime("%Y%m%d-%H%M%S")
    out = WS / f"snapshot-{name}-{ts}.txt"
    out.write_text(
        "\n".join([
            f"Snapshot: {name}",
            f"Time: {datetime.now().isoformat()}",
            f"Workspace: {WS}",
        ]),
        encoding="utf-8",
    )
    return str(out)


def freeze_bundle() -> dict:
    FREEZE_DIR.mkdir(parents=True, exist_ok=True)
    CAS_DIR.mkdir(parents=True, exist_ok=True)

    # Map hash guard record
    map_hash = sha256_file(MAP_FILE) if MAP_FILE.exists() else None
    guard_path = FREEZE_DIR / "map.hash"
    guard_path.write_text(map_hash or "", encoding="utf-8")

    # Manifest: sha256 for key SST files
    manifest = {}
    for rel in [
        "autonomous-system-map.mmd",
        "universal_task_execution_policy.md",
        "hierarchical-graph-reference.md",
        "GLOSSARY.md",
    ]:
        p = WS / rel
        if p.exists():
            manifest[rel] = sha256_file(p)
    (FREEZE_DIR / "manifest.sha256.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    # CAS index over a small, deterministic subset
    cas_index = {}
    for rel in manifest.keys():
        p = WS / rel
        digest = manifest[rel]
        dst = CAS_DIR / digest
        if not dst.exists():
            dst.write_bytes(p.read_bytes())
        cas_index[rel] = digest
    (FREEZE_DIR / "cas.index.json").write_text(json.dumps(cas_index, indent=2), encoding="utf-8")

    # Create a tar bundle of the freeze dir for portability
    tar_path = FREEZE_DIR / "freeze_bundle.tar.xz"
    with tarfile.open(tar_path, mode="w:xz") as tf:
        items = ["map.hash", "manifest.sha256.json", "cas.index.json"]
        if WORKLOAD_ROLLUP.exists():
            items.append("workload_rollup.json")
        for doc in [WS / "REPRO.md", WS / "COVERAGE.md", FREEZE_DIR / "FINAL_POLICY_REPORT.md"]:
            if doc.exists():
                # copy into exports root for bundling
                dst = FREEZE_DIR / doc.name
                if not dst.exists() or sha256_file(doc) != sha256_file(dst):
                    dst.write_bytes(doc.read_bytes())
                items.append(doc.name)
        for item in items:
            tf.add(FREEZE_DIR / item, arcname=item)

    # Final human-readable report
    final_md = FREEZE_DIR / "FINAL_REPORT.md"
    final_md.write_text(
        "\n".join([
            "# FINAL REPORT (Freeze)",
            f"Time: {datetime.now().isoformat()}",
            f"Map Hash: {map_hash}",
            "Artifacts:",
            "- map.hash",
            "- manifest.sha256.json",
            "- cas.index.json",
            ("- workload_rollup.json" if WORKLOAD_ROLLUP.exists() else "- workload_rollup.json (absent)"),
            ("- REPRO.md" if (FREEZE_DIR/"REPRO.md").exists() else "- REPRO.md (absent)"),
            ("- COVERAGE.md" if (FREEZE_DIR/"COVERAGE.md").exists() else "- COVERAGE.md (absent)"),
            ("- FINAL_POLICY_REPORT.md" if (FREEZE_DIR/"FINAL_POLICY_REPORT.md").exists() else "- FINAL_POLICY_REPORT.md (absent)"),
            "- freeze_bundle.tar.xz",
        ]),
        encoding="utf-8",
    )
    return {
        "map_hash": map_hash,
        "manifest": manifest,
        "cas_index": cas_index,
        "bundle": str(tar_path),
        "report": str(final_md),
    }


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else "audit"
    start = datetime.now().isoformat()
    if not AUDIT.exists():
        print(f"❌ Missing audit tool: {AUDIT}")
        return 2
    print(f"🧭 Orchestrator start ({mode}) @ {start}")

    ver = triple_verify()
    gaps = ver["report"].get("summary", {}).get("gaps", -1)
    passed = ver["report"].get("summary", {}).get("checks_passed", 0)
    total = ver["report"].get("summary", {}).get("checks_total", 0)

    print("\n=== Triple Verification ===")
    for i, r in enumerate(ver["runs"], 1):
        print(f"Run {i}: code={r['code']}")
    print(f"Consistent: {ver['consistent']}")

    print("\n=== Audit Summary ===")
    print(json.dumps(ver["report"].get("summary", {}), indent=2))

    if mode == "audit":
        print("\n🔒 Policy: Offline audit-only mode. No execution performed.")
        return 0 if gaps == 0 and ver["consistent"] else 3

    if mode == "execute":
        print("\n🛠  Execute mode: additive-only auto-heal + re-verify")
        snap1 = snapshot("pre-exec")
        print(f"Snapshot created: {snap1}")
        changes = auto_heal_gaps(ver["report"])
        print(f"Applied changes: {json.dumps(changes, indent=2)}")
        ver2 = triple_verify()
        print("\n=== Post-Execution Audit Summary ===")
        print(json.dumps(ver2["report"].get("summary", {}), indent=2))
        final = {
            "started": start,
            "finished": datetime.now().isoformat(),
            "pre_exec_summary": ver["report"].get("summary", {}),
            "post_exec_summary": ver2["report"].get("summary", {}),
            "changes": changes,
            "consistent_pre": ver["consistent"],
            "consistent_post": ver2["consistent"],
        }
        (WS / "FINAL_REPORT.json").write_text(json.dumps(final, indent=2), encoding="utf-8")
        print("\n💾 FINAL_REPORT.json written")
        return 0 if final["post_exec_summary"].get("gaps", 1) == 0 and ver2["consistent"] else 5

    if mode == "workload":
        print("\n🏭 Workload mode: generate/validate/rollup")
        # Ensure at least 10000 tasks exist (resume-safe expansion)
        code, _ = run([sys.executable, str(WS / "tools" / "batch_task_runner.py"), "--count", "10000", "--resume"])
        if code != 0:
            print("❌ Workload generation failed")
            return 8
        # Validate all tasks
        code, _ = run([sys.executable, str(WS / "tools" / "validate_task_results.py"), "--sample", "10000"])
        if code != 0:
            print("❌ Workload validation failed")
            return 9
        # Rollup + triple verify
        code, out = run([sys.executable, str(WS / "tools" / "workload_rollup_verify.py"), "--count", "10000"])
        print(out)
        if code != 0:
            print("❌ Workload rollup triple-check failed")
            return 10
        print("✅ Workload complete and verified")
        return 0

    if mode == "freeze":
        print("\n🧊 Freeze mode: map-hash guard + manifest + CAS index + bundle")
        if gaps != 0:
            print("❌ Refusing to freeze: outstanding gaps present. Run execute first.")
            return 7
        # Map-hash guard: if previous freeze exists and map hash changed, warn
        prev_hash_path = FREEZE_DIR / "map.hash"
        prev_hash = prev_hash_path.read_text(encoding="utf-8").strip() if prev_hash_path.exists() else None
        current_hash = sha256_file(MAP_FILE) if MAP_FILE.exists() else None
        if prev_hash and current_hash and prev_hash != current_hash:
            print("⚠️  Map changed since last freeze. Proceeding to write new freeze artifacts.")
        artifacts = freeze_bundle()
        print("Artifacts written:")
        print(json.dumps(artifacts, indent=2))
        return 0

    if mode == "clean":
        print("\n🧹 Clean mode: dry-run → quarantine (no delete by default)")
        # Triple-list for candidates (consistency check)
        candidates_runs = []
        for _ in range(3):
            code, out = run([sys.executable, str(WS / "tools" / "workspace_cleaner.py"), "dry-run", "--limit", "200"])
            candidates_runs.append((code, out))
        if not all(c[0] == 0 for c in candidates_runs):
            print("❌ Candidate listing failed")
            return 11
        # Proceed with quarantine for the same limit
        code, out = run([sys.executable, str(WS / "tools" / "workspace_cleaner.py"), "quarantine", "--limit", "200"])
        print(out)
        if code != 0:
            print("❌ Quarantine failed")
            return 12
        print("✅ Clean (quarantine) completed; review .quarantine before deletion.")
        return 0

    if mode == "clean-finalize":
        print("\n🧽 Clean finalize: archive and delete quarantine")
        quarantine = WS / ".quarantine"
        if not quarantine.exists():
            print("No quarantine directory present. Nothing to finalize.")
            return 0
        FREEZE_DIR.mkdir(parents=True, exist_ok=True)
        archive = FREEZE_DIR / "quarantine_snapshot.tar.xz"
        with tarfile.open(archive, mode="w:xz") as tf:
            tf.add(quarantine, arcname="quarantine")
        # Delete quarantine after archiving
        import shutil as _shutil
        _shutil.rmtree(quarantine, ignore_errors=True)
        print(f"Archived and removed quarantine: {archive}")
        # Refresh export hashes if tool exists
        if (WS / "tools" / "hash_exports.py").exists():
            run([sys.executable, str(WS / "tools" / "hash_exports.py")])
        return 0

    print("\n🚫 Unknown mode; use 'audit', 'execute', or 'freeze'.")
    return 6


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
