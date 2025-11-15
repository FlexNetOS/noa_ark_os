#!/usr/bin/env python3
import argparse, os, shutil
def have(cmd): return shutil.which(cmd) is not None
ap = argparse.ArgumentParser(); ap.add_argument("--target", required=True); a = ap.parse_args()
recs = []
if a.target in {"build-wasm-host","run-wasm-demo","run-wasm-capfile"} and not have("cargo"):
    recs.append("Install Rust toolchain to build/run WASM host.")
if a.target in {"run-wasm-demo","run-wasm-capfile","mint-cap"} and "FLEX_CONNECTOR_SECRET" not in os.environ:
    recs.append("Set FLEX_CONNECTOR_SECRET to a strong secret; re-mint caps.")
if a.target in {"run-wasm-capfile"} and "FLEX_PREOPEN_DIR" not in os.environ:
    recs.append("Set FLEX_PREOPEN_DIR to the directory you want the connector to read (it will mount at /cap).")
if a.target in {"update-verity-policy","check-verity-policy"} and not have("fsverity"):
    recs.append("Install fsverity-utils; policy uses measured digests when available.")
if a.target in {"run-core"} and os.environ.get("FLEX_ENFORCE_MOUNT_RO","0") != "1":
    recs.append("Set FLEX_ENFORCE_MOUNT_RO=1 to require artifacts/ be mounted read-only.")
if a.target in {"sign","verify","all","init"} and "FLEX_MINISIGN_PUB" not in os.environ:
    recs.append("Export FLEX_MINISIGN_PUB=/path/to/minisign.pub for runtime signature enforcement.")
print("[next] Recommendations:" if recs else "[next] No immediate upgrades detected.")
for r in recs: print("  -", r)
