
#!/usr/bin/env python3
import argparse, os, sys
from pathlib import Path

# Minimal test that golden sample files exist and are non-empty.
# If capnp tools are installed, could extend to compile & validate.

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--contracts", required=True)
    ap.add_argument("--samples", required=True)
    args = ap.parse_args()

    contracts = Path(args.contracts)
    samples = Path(args.samples)

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

if __name__ == "__main__":
    main()
