
#!/usr/bin/env python3
import argparse, json, hashlib, sys
from pathlib import Path

def sha256_file(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for ch in iter(lambda: f.read(1<<20), b""):
            h.update(ch)
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
        if not line.strip(): continue
        h, name = line.split("  ", 1)
        manifest[name] = h

    errors = 0
    for name, h in manifest.items():
        p = root / name
        if not p.exists():
            print("[verify] missing:", name, file=sys.stderr); errors += 1; continue
        h2 = sha256_file(p)
        if h2 != h:
            print("[verify] hash mismatch:", name, file=sys.stderr); errors += 1

    if errors:
        sys.exit(2)
    print("[verify] OK")

if __name__ == "__main__":
    main()
