#!/usr/bin/env python3
import argparse, json
from pathlib import Path
def readb(p): return open(p,"rb").read()
def score(b: bytes)->int: return sum(b)%1000
def main():
    ap=argparse.ArgumentParser(); ap.add_argument("--tri", required=True); ap.add_argument("--parent", required=True); ap.add_argument("--report", required=True)
    a=ap.parse_args(); tri=Path(a.tri); parent=Path(a.parent); outdir=parent/"model-D"; outdir.mkdir(parents=True, exist_ok=True)
    report={"items":[]}
    for p in tri.glob("*.A.out"):
        stem=p.name[:-6]; A=readb(tri/f"{stem}.A.out"); B=readb(tri/f"{stem}.B.out"); C=readb(tri/f"{stem}.C.out")
        if A==B or A==C: win,src=A,"A"
        elif B==C: win,src=B,"B"
        else:
            sc={"A":score(A),"B":score(B),"C":score(C)}; src=max(sc,key=sc.get); win={"A":A,"B":B,"C":C}[src]
        (outdir/f"{stem}.out").write_bytes(win); report["items"].append({"input":stem,"winner":src,"output":str((outdir/f'{stem}.out').as_posix())})
    Path(a.report).write_text(json.dumps(report, indent=2)); print(f"[merge] {len(report['items'])} items")
if __name__=="__main__": main()
