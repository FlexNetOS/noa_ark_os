#!/usr/bin/env python3
import argparse, os, shutil
def have(cmd): return shutil.which(cmd) is not None
ap = argparse.ArgumentParser(); ap.add_argument("--target", required=True); a = ap.parse_args()
recs = []
if a.target in {"build-wasm-host","run-wasm-demo"} and not have("cargo"):
    recs.append("Install Rust toolchain (cargo) to build wasm host.")
if a.target in {"run-wasm-demo","mint-cap"} and "FLEX_CONNECTOR_SECRET" not in os.environ:
    recs.append("Set FLEX_CONNECTOR_SECRET to a strong secret before minting/running capability tokens.")
if a.target in {"fs-verity-enable","fs-verity-sign"} and not have("fsverity"):
    recs.append("Install fsverity-utils and ensure kernel supports fs-verity.")
if a.target in {"run-core"} and os.environ.get("FLEX_ENFORCE_MOUNT_RO","0") != "1":
    recs.append("Set FLEX_ENFORCE_MOUNT_RO=1 to require artifacts/ mount be read-only.")
if a.target in {"sign","verify","all","init"} and "FLEX_MINISIGN_PUB" not in os.environ:
    recs.append("Export FLEX_MINISIGN_PUB=/path/to/minisign.pub to enforce sig verification at runtime.")
print("[next] Recommendations:" if recs else "[next] No immediate upgrades detected.")
for r in recs: print("  -", r)
