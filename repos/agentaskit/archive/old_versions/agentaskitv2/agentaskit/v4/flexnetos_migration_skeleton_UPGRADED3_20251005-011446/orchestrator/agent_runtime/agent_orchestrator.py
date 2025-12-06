#!/usr/bin/env python3
import argparse, json, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; STATE=ROOT/"orchestrator"/"state"; LOG=STATE/"event_log.jsonl"; PLANS=STATE/"plans"; TOKENS=STATE/"tokens"; SSA_CAP=50
def now(): import time; return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
def write(p, obj): p.parent.mkdir(parents=True, exist_ok=True); p.write_text(json.dumps(obj, indent=2))
def get_pool(ssa): p=TOKENS/f"{ssa}.json"; 
# ensure pool
def get_pool(ssa):
    p=TOKENS/f"{ssa}.json"; p.parent.mkdir(parents=True, exist_ok=True)
    if not p.exists(): write(p, {"ssa":ssa,"cap":SSA_CAP,"in_use":0,"available":SSA_CAP})
    return json.loads(p.read_text())
def set_pool(ssa,pool): write(TOKENS/f"{ssa}.json", pool)
def append(ev): LOG.parent.mkdir(parents=True, exist_ok=True); LOG.open("a",encoding="utf-8").write(json.dumps(ev)+"\n")
def demo():
    pid="demo"; ssa="ssa-demo"
    write(PLANS/f"{pid}.json", {"id":pid,"version":1,"ssa":ssa,"nodes":{"A":{"status":"done"},"B":{"status":"done"},"C":{"status":"done"}}})
    set_pool(ssa, {"ssa":ssa,"cap":SSA_CAP,"in_use":0,"available":SSA_CAP})
    append({"ts":now(),"type":"plan-create","plan":pid,"ssa":ssa}); print("[demo] orchestrator state initialized")
if __name__=="__main__": demo()
