
#!/usr/bin/env python3
import argparse, os, subprocess, json, time
from pathlib import Path

def run_cmd(cmd, cwd, inp, outp, trace):
    t0 = time.time()
    env = os.environ.copy()
    env["TASK_INPUT"] = str(inp)
    with open(outp, "wb") as outf, open(trace, "w", encoding="utf-8") as tr:
        proc = subprocess.run(cmd, cwd=cwd, env=env, stdout=outf, stderr=subprocess.PIPE, shell=True, check=False)
        tr.write(json.dumps({
            "cmd": cmd, "cwd": str(cwd), "input": str(inp), "output": str(outp),
            "rc": proc.returncode, "stderr": proc.stderr.decode("utf-8","ignore"),
            "duration_ms": int((time.time()-t0)*1000)
        }, indent=2))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--inputs", required=True)
    ap.add_argument("--tri", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    inputs = Path(args.inputs); tri = Path(args.tri); out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    for sb in ["A","B","C"]:
        if not (tri/sb/"run.sh").exists():
            raise SystemExit(f"[tri-run] missing {tri/sb}/run.sh")
    for inp in inputs.glob("*"):
        if not inp.is_file(): continue
        for sb in ["A","B","C"]:
            run_cmd("./run.sh", tri/sb, inp, out/f"{inp.name}.{sb}.out", out/f"{inp.name}.{sb}.trace.json")
    print("[tri-run] done")

if __name__ == "__main__":
    main()
