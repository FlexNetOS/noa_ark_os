
#!/usr/bin/env python3
import argparse, json, hashlib
from pathlib import Path

def read_bytes(p: Path):
    with open(p, "rb") as f:
        return f.read()

def fitness_score(b: bytes) -> int:
    # Placeholder: deterministic, content-based score for reproducibility.
    # Replace with real contract tests + p99 latency + correctness checker.
    return sum(b) % 1000

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tri", required=True)      # sandbox/outputs
    ap.add_argument("--parent", required=True)   # sandbox/parent
    ap.add_argument("--report", required=True)
    args = ap.parse_args()

    tri = Path(args.tri)
    parent = Path(args.parent)
    outdir = parent / "model-D"
    outdir.mkdir(parents=True, exist_ok=True)

    grouped = {}
    for p in tri.glob("*.A.out"):
        stem = p.name[:-6]  # remove .A.out
        A = tri / f"{stem}.A.out"
        B = tri / f"{stem}.B.out"
        C = tri / f"{stem}.C.out"
        if A.exists() and B.exists() and C.exists():
            grouped[stem] = (A,B,C)

    report = {"items": [], "summary": {}}
    for stem,(A,B,C) in grouped.items():
        a = read_bytes(A); b = read_bytes(B); c = read_bytes(C)
        # Majority vote
        if a==b or a==c:
            winner = a
            source = "A" if a==b or a==c else "B"
        elif b==c:
            winner = b; source="B"
        else:
            # Fitness pick
            scores = {"A": fitness_score(a),"B": fitness_score(b),"C": fitness_score(c)}
            source = max(scores, key=scores.get)
            winner = {"A":a,"B":b,"C":c}[source]
        outp = outdir / f"{stem}.out"
        with open(outp, "wb") as f:
            f.write(winner)
        report["items"].append({"input": stem, "winner": source, "output": str(outp)})

    report["summary"]["total"] = len(report["items"])
    Path(args.report).write_text(json.dumps(report, indent=2))
    print(f"[merge] wrote {args.report} and {len(report['items'])} outputs into {outdir}")

if __name__ == "__main__":
    main()
