
#!/usr/bin/env python3
import argparse, os, subprocess, json, time
from pathlib import Path

def run_cmd(cmd, cwd, input_path, out_path, trace_path):
    t0 = time.time()
    env = os.environ.copy()
    env["TASK_INPUT"] = str(input_path)
    with open(out_path, "wb") as outf, open(trace_path, "w", encoding="utf-8") as trace:
        proc = subprocess.run(cmd, cwd=cwd, env=env, stdout=outf, stderr=subprocess.PIPE, shell=True, check=False)
        t1 = time.time()
        trace.write(json.dumps({
            "cmd": cmd,
            "cwd": str(cwd),
            "input": str(input_path),
            "output": str(out_path),
            "rc": proc.returncode,
            "stderr": proc.stderr.decode("utf-8", "ignore"),
            "duration_ms": int((t1-t0)*1000)
        }, indent=2))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--inputs", required=True)
    ap.add_argument("--tri", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    root = Path(args.root)
    inputs = Path(args.inputs)
    tri = Path(args.tri)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    sandboxes = {"A": tri/"A", "B": tri/"B", "C": tri/"C"}
    for k, sb in sandboxes.items():
        if not (sb/"run.sh").exists():
            raise SystemExit(f"[tri-run] missing {sb}/run.sh")

    for inp in inputs.glob("*"):
        if not inp.is_file(): continue
        for k, sb in sandboxes.items():
            name = f"{inp.name}.{k}.out"
            trace = f"{inp.name}.{k}.trace.json"
            run_cmd("./run.sh", sb, inp, out/name, out/trace)

    print("[tri-run] done.")

if __name__ == "__main__":
    main()
