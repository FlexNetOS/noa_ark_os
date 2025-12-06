
#!/usr/bin/env python3
import argparse, json, os, hashlib
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
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    root = Path(args.root)
    sbom = json.loads(Path(args.sbom).read_text())
    lines = []
    for comp in sbom.get("components", []):
        name = comp["name"]
        fpath = root / name
        if fpath.is_file():
            lines.append(f"{sha256_file(fpath)}  {name}")
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text("\n".join(lines) + "\n")
    print(f"[signer] wrote manifest {args.out} ({len(lines)} entries).")

if __name__ == "__main__":
    main()
