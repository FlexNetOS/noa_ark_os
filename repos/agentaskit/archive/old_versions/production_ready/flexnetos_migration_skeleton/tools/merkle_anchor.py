
#!/usr/bin/env python3
import argparse, hashlib, json, os, time
from pathlib import Path

def sha256(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1<<20), b""):
            h.update(chunk)
    return h.digest()

def merkle_root(files):
    hashes = [sha256(p) for p in files]
    if not hashes:
        return None
    while len(hashes) > 1:
        it = iter(hashes)
        pairs = list(zip(it, it))
        if len(hashes) % 2 == 1:
            pairs.append((hashes[-1], hashes[-1]))
        hashes = [hashlib.sha256(a+b).digest() for a,b in pairs]
    return hashes[0].hex()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--sbom", required=True)
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    root = Path(args.root)
    release_set = [
        root / "Makefile",
        Path(args.sbom),
        Path(args.manifest),
    ]

    # include policies and contracts
    for p in (root/"orchestrator"/"policies").rglob("*"):
        if p.is_file(): release_set.append(p)
    for p in (root/"contracts").rglob("*"):
        if p.is_file(): release_set.append(p)

    mr = merkle_root(release_set) or ""
    meta = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "merkle_root": mr,
        "files": [str(p.relative_to(root)) for p in release_set]
    }
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(meta, indent=2))
    print(f"[anchor] wrote anchor file {args.out}")

if __name__ == "__main__":
    main()
