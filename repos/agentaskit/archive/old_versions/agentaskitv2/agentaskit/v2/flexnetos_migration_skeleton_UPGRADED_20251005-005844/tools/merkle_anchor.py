
#!/usr/bin/env python3
import argparse, hashlib, json, time
from pathlib import Path

def hfile(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for c in iter(lambda: f.read(1<<20), b""): h.update(c)
    return h.digest()

def merkle(files):
    nodes = [hfile(p) for p in files]
    if not nodes: return ""
    while len(nodes) > 1:
        it = iter(nodes); pairs = list(zip(it, it))
        if len(nodes) % 2 == 1: pairs.append((nodes[-1], nodes[-1]))
        nodes = [hashlib.sha256(a+b).digest() for a,b in pairs]
    return nodes[0].hex()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--sbom", required=True)
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    root = Path(args.root)
    release = [root/"Makefile", Path(args.sbom), Path(args.manifest)]
    for sub in ["orchestrator/policies", "contracts"]:
        for p in (root/sub).rglob("*"):
            if p.is_file(): release.append(p)

    mr = merkle(release)
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    meta = {"timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "merkle_root": mr,
            "files": [str(p.relative_to(root)) for p in release]}
    Path(args.out).write_text(json.dumps(meta, indent=2))
    print("[anchor] wrote", args.out)

if __name__ == "__main__":
    main()
