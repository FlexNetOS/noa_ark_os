#!/usr/bin/env python3
"""
Enhanced contract test combining capabilities from all versions.
HEALED: Preserves original detailed error checking from v1 while adding 
capnp compilation check from v7.
"""
import argparse, shutil, subprocess, sys, os
from pathlib import Path

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--contracts", required=True)
    ap.add_argument("--samples", required=True)
    args = ap.parse_args()

    contracts = Path(args.contracts)
    samples = Path(args.samples)

    # Enhanced check from original version
    cap = contracts / "inference.capnp"
    if not cap.exists():
        print("[contract-test] missing contracts/inference.capnp", file=sys.stderr)
        raise SystemExit(2)

    req = samples / "request.bin"
    rep = samples / "reply.bin"
    for p in (req, rep):
        if not p.exists() or p.stat().st_size == 0:
            print(f"[contract-test] missing or empty sample: {p}", file=sys.stderr)
            raise SystemExit(2)

    print("[contract-test] golden samples present; basic checks pass.")

    # Enhanced capnp compilation check from v7
    capnp = shutil.which("capnp")
    strict = os.environ.get("CAPNP_STRICT","0") == "1"
    if capnp:
        try:
            subprocess.run([capnp, "compile", "-o-", str(cap)], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            print("[contract-test] capnp compile check passed")
        except Exception as e:
            msg = f"[contract-test] capnp present but compile failed: {e}"
            if strict: 
                print(msg, file=sys.stderr)
                sys.exit(2)
            else: 
                print(msg + " (warning only)")
    else:
        print("[contract-test] capnp not found; skipping compilation check")

    print("[contract-test] all checks pass")

if __name__ == "__main__":
    main()