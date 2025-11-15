#!/usr/bin/env python3
import argparse, os, shutil
from pathlib import Path
def have(cmd): import shutil ; return shutil.which(cmd) is not None
def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--target", required=True); args = ap.parse_args()
    root = Path.cwd()
    manifest = root/"artifacts"/"MANIFEST.sha256"
    minisig = manifest.with_suffix(".sha256.minisig") if manifest.name.endswith(".sha256") else manifest.with_suffix(".minisig")
    recs = []
    if args.target in {"sign","verify","all","init"}:
        if not have("minisign"):
            recs.append("Install minisign and set FLEX_MINISIGN_PUB to enforce runtime signature checks.")
        if "FLEX_MINISIGN_PUB" not in os.environ:
            recs.append("Export FLEX_MINISIGN_PUB=/path/to/pubkey and re-run `make run-core`.")
        if not minisig.exists():
            recs.append("Create MANIFEST.sha256.minisig with `minisign -Sm artifacts/MANIFEST.sha256 -s orchestrator/keys/minisign.key`.")
    if args.target in {"seal-manifest","verify","run-core","build-core"}:
        if not have("fsverity"):
            recs.append("Install fsverity-utils; then `make fs-verity-enable`.")
        if not have("chattr"):
            recs.append("Install e2fsprogs to use immutable sealing (`make seal-manifest`).")
        if os.environ.get("FLEX_ENFORCE_SEAL","0") != "1":
            recs.append("Set FLEX_ENFORCE_SEAL=1 before `make run-core` to require immutable/fs-verity seal.")
    if args.target in {"contract-test","build-core"}:
        if os.environ.get("CAPNP_STRICT","0") != "1":
            recs.append("Set CAPNP_STRICT=1 to gate on Cap'n Proto IDL compile.")
    if args.target in {"tri-run","merge"}:
        recs.append("Replace byte-sum scoring with domain metrics in sandbox/tri-sandbox/unifier/merge.py.")
    if args.target in {"run-core","smoke-client","py-client"}:
        recs.append("Pin with `make numa-pin` and provision hugepages via `make hugepages`.")
    if not recs: print("[next] No immediate upgrades detected.")
    else:
        print("[next] Recommendations:"); 
        for r in recs: print("  -", r)
if __name__ == "__main__": main()
