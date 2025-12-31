#!/usr/bin/env python3
import argparse, hashlib, json, time
from pathlib import Path
def hfile(p):
    h=hashlib.sha256()
    with open(p,"rb") as f:
        for c in iter(lambda:f.read(1<<20), b""): h.update(c)
    return h.digest()
def merkle(files):
    hs=[hfile(p) for p in files]
    if not hs: return ""
    while len(hs)>1:
        it=iter(hs); pairs=list(zip(it,it)); 
        if len(hs)%2==1: pairs.append((hs[-1],hs[-1]))
        hs=[hashlib.sha256(a+b).digest() for a,b in pairs]
    return hs[0].hex()
def main():
    ap=argparse.ArgumentParser(); ap.add_argument("--root", required=True); ap.add_argument("--sbom", required=True); ap.add_argument("--manifest", required=True); ap.add_argument("--out", required=True)
    a=ap.parse_args(); root=Path(a.root)
    release=[root/"Makefile", Path(a.sbom), Path(a.manifest)]
    for sub in ["orchestrator/policies","contracts"]:
        for p in (root/sub).rglob("*"):
            if p.is_file(): release.append(p)
    mr=merkle(release); Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps({"timestamp":time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),"merkle_root":mr,"files":[str(p.relative_to(root)) for p in release]}, indent=2))
    print("[anchor] wrote", a.out)
if __name__=="__main__": main()
