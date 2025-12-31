
#!/usr/bin/env python3
import argparse, json, shutil, time
from pathlib import Path

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parent", required=True)
    ap.add_argument("--exec", required=True)
    args = ap.parse_args()

    src = Path(args.parent) / "model-D"
    dst = Path(args.exec) / "models" / "model-D"
    dst.parent.mkdir(parents=True, exist_ok=True)
    if not src.exists():
        raise SystemExit("[promote] nothing to promote; run merge first.")
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)

    att = Path(args.exec) / "attestation" / f"promote-{int(time.time())}.json"
    att.parent.mkdir(parents=True, exist_ok=True)
    att.write_text(json.dumps({
        "at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "source": str(src),
        "dest": str(dst),
        "policy_hash": "TODO:compute",
        "sbom_ref": "../../sbom/sbom.cdx.json"
    }, indent=2))
    print(f"[promote] promoted to {dst} with attestation {att}")

if __name__ == "__main__":
    main()
