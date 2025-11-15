#!/usr/bin/env python3
"""
HEALED: fs_integrity.sh functionality that was missing in v7
Preserves file system integrity operations from v5.
"""
import argparse, shutil, subprocess, sys, os
from pathlib import Path

def run_cmd(cmd):
    """Run shell command and return result"""
    try:
        result = subprocess.run(cmd, shell=True, check=True, capture_output=True, text=True)
        return result.stdout.strip(), True
    except subprocess.CalledProcessError as e:
        return e.stderr.strip(), False

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", required=True, help="File to operate on")
    ap.add_argument("--operation", required=True, 
                   choices=['seal-immutable', 'unseal-immutable', 'verity-enable', 'verity-status', 'verity-sign'],
                   help="File system integrity operation")
    ap.add_argument("--key", help="Key file for verity-sign")
    ap.add_argument("--cert", help="Certificate file for verity-sign")
    args = ap.parse_args()

    target = Path(args.target)
    if not target.exists():
        print(f"[fs_integrity] target file {target} not found", file=sys.stderr)
        sys.exit(1)

    if args.operation == "seal-immutable":
        if not shutil.which("chattr"):
            print("[fs_integrity] chattr not found", file=sys.stderr)
            sys.exit(1)
        out, ok = run_cmd(f"chattr +i '{target}'")
        if ok:
            print(f"[fs_integrity] immutable set on {target}")
        else:
            print(f"[fs_integrity] failed to set immutable: {out}", file=sys.stderr)
            sys.exit(1)

    elif args.operation == "unseal-immutable":
        if not shutil.which("chattr"):
            print("[fs_integrity] chattr not found", file=sys.stderr)
            sys.exit(1)
        out, ok = run_cmd(f"chattr -i '{target}'")
        if ok:
            print(f"[fs_integrity] immutable cleared on {target}")
        else:
            print(f"[fs_integrity] failed to clear immutable: {out}", file=sys.stderr)
            sys.exit(1)

    elif args.operation == "verity-enable":
        if not shutil.which("fsverity"):
            print("[fs_integrity] fsverity tool not found", file=sys.stderr)
            sys.exit(1)
        out, ok = run_cmd(f"fsverity enable '{target}' --hash-alg sha256")
        if ok:
            print(f"[fs_integrity] fs-verity enabled on {target}")
        else:
            print(f"[fs_integrity] fsverity enable failed: {out}", file=sys.stderr)
            sys.exit(1)

    elif args.operation == "verity-status":
        if shutil.which("fsverity"):
            out, ok = run_cmd(f"fsverity measure '{target}'")
            if ok:
                print(f"[fs_integrity] fsverity measure: {out}")
            else:
                print(f"[fs_integrity] fsverity measure failed: {out}")
        else:
            print("[fs_integrity] fsverity tool not found")
        
        if shutil.which("lsattr"):
            out, ok = run_cmd(f"lsattr '{target}'")
            if ok:
                print(f"[fs_integrity] lsattr: {out}")

    elif args.operation == "verity-sign":
        key_file = args.key or os.environ.get("FSV_KEY", "/path/key.pem")
        cert_file = args.cert or os.environ.get("FSV_CERT", "/path/cert.pem")
        
        if not Path(key_file).exists() or not Path(cert_file).exists():
            print(f"[fs_integrity] key or cert file not found: {key_file}, {cert_file}", file=sys.stderr)
            sys.exit(1)
            
        out, ok = run_cmd(f"fsverity sign '{target}' '{key_file}' '{cert_file}'")
        if ok:
            print(f"[fs_integrity] fs-verity signed {target}")
        else:
            print(f"[fs_integrity] fs-verity sign failed: {out}", file=sys.stderr)
            sys.exit(1)

if __name__ == "__main__":
    main()