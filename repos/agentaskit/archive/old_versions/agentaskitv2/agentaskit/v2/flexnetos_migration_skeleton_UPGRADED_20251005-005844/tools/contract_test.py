
#!/usr/bin/env python3
import argparse, sys
from pathlib import Path

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--contracts", required=True)
    ap.add_argument("--samples", required=True)
    args = ap.parse_args()

    cap = Path(args.contracts)/"inference.capnp"
    rq = Path(args.samples)/"request.bin"
    rp = Path(args.samples)/"reply.bin"
    ok = cap.exists() and rq.exists() and rq.stat().st_size>0 and rp.exists() and rp.stat().st_size>0
    if not ok:
        print("[contract-test] missing IDL or golden samples", file=sys.stderr)
        sys.exit(2)
    print("[contract-test] basic checks pass")

if __name__ == "__main__":
    main()
