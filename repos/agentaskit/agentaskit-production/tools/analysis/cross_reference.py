#!/usr/bin/env python3
import argparse, hashlib, json, os, re, sys
from pathlib import Path

ARCHIVE_PATTERNS = [
    re.compile(r".*archive[/\\]old_versions[/\\].*", re.IGNORECASE),
    re.compile(r".*[/\\](V[2-7]|v[2-7]|agentaskitv[2-7]).*", re.IGNORECASE),
]

IGNORE_DIRS = {".git", ".github", ".venv", "node_modules", "target", ".idea", ".vs", ".vscode", "__pycache__"}


def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def is_textual(p: Path) -> bool:
    try:
        with p.open("rb") as f:
            b = f.read(2048)
        return b.find(b"\0") == -1
    except Exception:
        return False


def walk_files(root: Path):
    for dirpath, dirnames, filenames in os.walk(root):
        # prune ignored dirs
        dirnames[:] = [d for d in dirnames if d not in IGNORE_DIRS]
        for fn in filenames:
            fp = Path(dirpath) / fn
            yield fp


def classify_path(p: Path) -> str:
    s = str(p.as_posix())
    for rx in ARCHIVE_PATTERNS:
        if rx.match(s):
            return "archive"
    # Explicit final deploy package: agentaskit-production subtree
    if "/agentaskit-production/" in s or s.startswith("agentaskit-production/") or s.endswith("agentaskit-production"):
        return "production"
    return "other"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="repo root")
    ap.add_argument("--out-dir", required=True, help="output dir for artifacts")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    outdir = Path(args.out_dir)
    outdir.mkdir(parents=True, exist_ok=True)

    manifest = {
        "root": str(root),
        "summary": {"files_total": 0, "archive_files": 0, "production_files": 0, "other_files": 0},
        "files": [],
    }

    for fp in walk_files(root):
        try:
            kind = classify_path(fp)
            h = sha256_file(fp)
            rel = fp.relative_to(root)
            entry = {
                "path": str(rel).replace("\\", "/"),
                "kind": kind,
                "sha256": h,
                "size": fp.stat().st_size,
                "text": is_textual(fp),
            }
            manifest["files"].append(entry)
            manifest["summary"]["files_total"] += 1
            if kind == "archive":
                manifest["summary"]["archive_files"] += 1
            elif kind == "production":
                manifest["summary"]["production_files"] += 1
            else:
                manifest["summary"]["other_files"] += 1
        except Exception as e:
            print(f"WARN: {fp}: {e}", file=sys.stderr)

    # lineage mapping: same filename at different roots with differing hashes
    by_name = {}
    for f in manifest["files"]:
        key = os.path.basename(f["path"]).lower()
        by_name.setdefault(key, []).append(f)

    lineage = []
    for name, entries in by_name.items():
        if len(entries) < 2:
            continue
        kinds = {e["kind"] for e in entries}
        if "archive" in kinds and "production" in kinds:
            lineage.append({"name": name, "entries": entries})

    # duplicates within tree (same sha, different paths)
    by_hash = {}
    for f in manifest["files"]:
        by_hash.setdefault(f["sha256"], []).append(f)
    duplicates = [v for v in by_hash.values() if len(v) > 1]

    # Production vs non-production comparison by basename
    prod_names = set()
    nonprod_names = set()
    for f in manifest["files"]:
        nm = os.path.basename(f["path"]).lower()
        if f["kind"] == "production":
            prod_names.add(nm)
        else:
            nonprod_names.add(nm)
    missing_in_production = sorted(list(nonprod_names - prod_names))[:200]
    extra_in_production = sorted(list(prod_names - nonprod_names))[:200]

    # High-level report
    report = {
        "summary": manifest["summary"],
        "lineage_pairs": len(lineage),
        "duplicates_groups": len(duplicates),
        "production_missing_dirs": [],
        "missing_in_production_basenames": missing_in_production,
        "extra_in_production_basenames": extra_in_production,
    }

    # heuristics: expected production components
    expected_dirs = [
        "agentaskit-production/core/src/workflows/seven_phase",
        "agentaskit-production/tests",
        "agentaskit-production/dashboards",
        "agentaskit-production/alerts",
        "agentaskit-production/slo",
        "agentaskit-production/security/policies",
        "agentaskit-production/operational_hash",
        "agentaskit-production/TEST",
        ".github/workflows",
    ]
    for d in expected_dirs:
        if not (root / d).exists():
            report["production_missing_dirs"].append(d)

    # write artifacts
    (outdir / "manifest.json").write_text(json.dumps(manifest, indent=2))
    (outdir / "report.json").write_text(json.dumps(report, indent=2))

    # write human markdown
    md = []
    md.append("# Cross-reference Report (archives V2–V7 → production)")
    s = manifest["summary"]
    md.append(f"- Files total: {s['files_total']} (archive: {s['archive_files']}, production: {s['production_files']}, other: {s['other_files']})")
    md.append(f"- Lineage pairs (archive↔production filename collisions): {len(lineage)}")
    md.append(f"- Duplicate groups (identical sha across different paths): {len(duplicates)}")
    if report["production_missing_dirs"]:
        md.append("## Missing expected production components")
        for m in report["production_missing_dirs"]:
            md.append(f"- {m}")
    if missing_in_production:
        md.append("## Basenames present outside production but not in final package (top 200)")
        for n in missing_in_production[:50]:
            md.append(f"- {n}")
        if len(missing_in_production) > 50:
            md.append(f"… and {len(missing_in_production)-50} more")
    if extra_in_production:
        md.append("## Basenames present in final package but not elsewhere (top 200)")
        for n in extra_in_production[:50]:
            md.append(f"- {n}")
        if len(extra_in_production) > 50:
            md.append(f"… and {len(extra_in_production)-50} more")
    if lineage:
        md.append("## Archive ↔ Production lineage examples (up to 10)")
        for pair in lineage[:10]:
            md.append(f"- {pair['name']}:")
            for e in pair["entries"]:
                md.append(f"  - {e['kind']}: {e['path']} ({e['sha256'][:12]} … size {e['size']})")
    if duplicates:
        md.append("## Duplicate content groups (up to 5)")
        for group in duplicates[:5]:
            md.append(f"- sha {group[0]['sha256'][:12]} present in:")
            for e in group:
                md.append(f"  - {e['path']}")
    (outdir / "report.md").write_text("\n".join(md), encoding="utf-8")

if __name__ == "__main__":
    sys.exit(main())
