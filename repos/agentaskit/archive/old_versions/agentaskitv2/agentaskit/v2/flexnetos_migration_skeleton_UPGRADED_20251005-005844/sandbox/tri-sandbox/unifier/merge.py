
#!/usr/bin/env python3
import argparse, json
from pathlib import Path

def readb(p): return open(p,"rb").read()

def score(b: bytes) -> int:
    # deterministic placeholder; replace with correctness checks
    return sum(b) % 1000

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tri", required=True)
    ap.add_argument("--parent", required=True)
    ap.add_argument("--report", required=True)
    args = ap.parse_args()

    tri = Path(args.tri); parent = Path(args.parent); outdir = parent/"model-D"
    outdir.mkdir(parents=True, exist_ok=True)

    report = {"items": []}
    stems = set([p.name[:-6] for p in tri.glob("*.A.out")])
    for s in stems:
        A = readb(tri/f"{s}.A.out"); B = readb(tri/f"{s}.B.out"); C = readb(tri/f"{s}.C.out")
        if A==B or A==C: win, src = A, "A"
        elif B==C: win, src = B, "B"
        else:
            scores = {"A":score(A),"B":score(B),"C":score(C)}
            src = max(scores, key=scores.get); win = {"A":A,"B":B,"C":C}[src]
        with open(outdir/f"{s}.out","wb") as f: f.write(win)
        report["items"].append({"input": s, "winner": src, "output": str((outdir/f'{s}.out').as_posix())})

    Path(args.report).write_text(json.dumps(report, indent=2))
    print(f"[merge] wrote {args.report} with {len(report['items'])} items")

if __name__ == "__main__":
    main()
