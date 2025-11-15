#!/usr/bin/env python3
import argparse, json, hashlib, time
from pathlib import Path
def sha256_file(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1<<20), b""): h.update(chunk)
    return h.hexdigest()
def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--root", required=True); ap.add_argument("--out", required=True)
    args = ap.parse_args(); root = Path(args.root)
    comps = []
    for p in root.rglob("*"):
        if p.is_file() and not str(p).startswith(str(root/"anchors")):
            comps.append({"type":"file","name":str(p.relative_to(root)),"hashes":[{"alg":"SHA-256","content":sha256_file(p)}],"version":"1.0.0"})
    sbom={"bomFormat":"CycloneDX","specVersion":"1.5","serialNumber":"urn:uuid:offline","version":1,"metadata":{"timestamp":time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())},"components":comps}
    Path(args.out).parent.mkdir(parents=True, exist_ok=True); Path(args.out).write_text(json.dumps(sbom, indent=2))
    print(f"[sbom_gen] {len(comps)} components -> {args.out}")
if __name__=="__main__": main()
