#!/usr/bin/env python3
import argparse, json, subprocess, hashlib, shutil
from pathlib import Path

def fsverity_measure(path: Path):
    if shutil.which("fsverity"):
        try:
            out = subprocess.run(["fsverity", "measure", str(path)], check=True, capture_output=True, text=True).stdout
            for tok in out.replace("\n"," ").split():
                if tok.startswith("sha256:"):
                    return tok.split("sha256:")[-1]
        except Exception:
            pass
    # Fallback: file sha256 (not the verity hash); flagged
    h=hashlib.sha256()
    with open(path,"rb") as f:
        for ch in iter(lambda:f.read(1<<20), b""): h.update(ch)
    return "filesha256:"+h.hexdigest()

def main():
    ap=argparse.ArgumentParser()
    sub=ap.add_subparsers(dest="cmd", required=True)
    up = sub.add_parser("update"); up.add_argument("--file", required=True); up.add_argument("--policy", required=True)
    ck = sub.add_parser("check");  ck.add_argument("--file", required=True); ck.add_argument("--policy", required=True)
    a=ap.parse_args()
    f=Path(a.file); pol=Path(a.policy); pol.parent.mkdir(parents=True, exist_ok=True)
    if a.cmd=="update":
        digest = fsverity_measure(f)
        policy = {"allow": {str(f): digest}, "note": "fs-verity measured digests; if prefix 'filesha256:' used, upgrade to fs-verity."}
        pol.write_text(json.dumps(policy, indent=2))
        print("[verity] updated", pol)
    else:
        if not pol.exists(): raise SystemExit("[verity] policy missing")
        policy=json.loads(pol.read_text()); allow=policy.get("allow",{})
        expect=allow.get(str(f))
        got=fsverity_measure(f)
        if expect is None: raise SystemExit("[verity] file not in policy")
        if got != expect: raise SystemExit(f"[verity] mismatch: expected {expect}, got {got}")
        print("[verity] policy check OK")
if __name__=="__main__": main()
