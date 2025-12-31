
#!/usr/bin/env python3
import argparse, json, hashlib
from pathlib import Path

def sha256_file(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1024*1024), b""):
            h.update(chunk)
    return h.hexdigest()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--sbom", required=True)
    ap.add_argument("--manifest", required=True)
    args = ap.parse_args()
    root = Path(args.root)

    manifest = {}
    for line in Path(args.manifest).read_text().splitlines():
        if not line.strip():
            continue
        h, name = line.split("  ", 1)
        manifest[name] = h

    mismatches = []
    for name, h in manifest.items():
        p = root / name
        if not p.exists():
            mismatches.append((name, "missing"))
            continue
        h2 = sha256_file(p)
        if h2 != h:
            mismatches.append((name, "hash-mismatch"))

    if mismatches:
        for m in mismatches:
            print("[verify] ERROR:", m[0], m[1])
        raise SystemExit(2)
    print("[verify] OK")

if __name__ == "__main__":
    main()
