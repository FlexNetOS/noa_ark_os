#!/usr/bin/env python3
import argparse, json, hashlib
from pathlib import Path
def sha256_file(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for ch in iter(lambda: f.read(1<<20), b""): h.update(ch)
    return h.hexdigest()
def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--root", required=True); ap.add_argument("--sbom", required=True); ap.add_argument("--out", required=True)
    args=ap.parse_args(); root=Path(args.root); sbom=json.loads(Path(args.sbom).read_text())
    lines=[]
    for comp in sbom.get("components", []):
        p=root/comp["name"]
        if p.is_file(): lines.append(f"{sha256_file(p)}  {comp['name']}")
    Path(args.out).parent.mkdir(parents=True, exist_ok=True); Path(args.out).write_text("\n".join(lines)+"\n")
    print(f"[signer] manifest -> {args.out} ({len(lines)} files)")
if __name__=="__main__": main()
