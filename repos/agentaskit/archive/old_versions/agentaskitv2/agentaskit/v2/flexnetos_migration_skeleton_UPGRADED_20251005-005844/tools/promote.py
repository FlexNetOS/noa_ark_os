
#!/usr/bin/env python3
import argparse, json, shutil, time
from pathlib import Path

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parent", required=True)
    ap.add_argument("--exec", required=True)
    args = ap.parse_args()

    src = Path(args.parent)/"model-D"
    dst = Path(args.exec)/"models"/"model-D"
    if not src.exists(): raise SystemExit("[promote] nothing to promote")
    if dst.exists(): shutil.rmtree(dst)
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(src, dst)
    att = Path(args.exec)/"attestation"/f"promote-{int(time.time())}.json"
    att.parent.mkdir(parents=True, exist_ok=True)
    att.write_text(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                               "source": str(src), "dest": str(dst)}, indent=2))
    print("[promote] promoted", dst)

if __name__ == "__main__":
    main()
