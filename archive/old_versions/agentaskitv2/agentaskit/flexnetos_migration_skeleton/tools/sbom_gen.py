
#!/usr/bin/env python3
import argparse, json, os, hashlib, time
from pathlib import Path

def sha256_file(p: Path):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1024*1024), b""):
            h.update(chunk)
    return h.hexdigest()

def collect_components(root: Path):
    components = []
    for p in root.rglob("*"):
        if p.is_file():
            rel = str(p.relative_to(root))
            # omit big binaries in this minimal example? keep all but anchors
            if rel.startswith("anchors/"):
                continue
            hashv = sha256_file(p)
            components.append({
                "type": "file",
                "name": rel,
                "hashes": [{"alg": "SHA-256", "content": hashv}],
                "version": "1.0.0"
            })
    return components

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    root = Path(args.root)

    sbom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": f"urn:uuid:{os.urandom(16).hex()}",
        "version": 1,
        "metadata": {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "tools": [{"vendor": "FlexNetOS", "name": "sbom_gen", "version": "0.1"}]
        },
        "components": collect_components(root)
    }
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(sbom, f, indent=2)
    print(f"[sbom_gen] wrote {args.out} with {len(sbom['components'])} components.")

if __name__ == "__main__":
    main()
