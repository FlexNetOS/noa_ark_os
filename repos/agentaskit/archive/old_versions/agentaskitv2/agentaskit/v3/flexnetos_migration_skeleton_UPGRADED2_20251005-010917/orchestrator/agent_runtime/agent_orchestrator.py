#!/usr/bin/env python3
import argparse, json, time, uuid, sys
from pathlib import Path
from dataclasses import dataclass, asdict
ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT/"orchestrator"/"state"; LOG=STATE/"event_log.jsonl"; PLANS=STATE/"plans"; TOKENS=STATE/"tokens"; SSA_CAP=50
def now(): return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
def append_event(ev): LOG.parent.mkdir(parents=True, exist_ok=True); LOG.open("a",encoding="utf-8").write(json.dumps(ev)+"\n")
@dataclass
class Node: id:str; spec:dict; status:str="ready"; budget:dict=None; parents:list=None; children:list=None; attempts:int=0
@dataclass
class Plan: id:str; version:int; nodes:dict; ssa:str
def save_plan(p): PLANS.mkdir(parents=True, exist_ok=True); (PLANS/f"{p.id}.json").write_text(json.dumps({"id":p.id,"version":p.version,"ssa":p.ssa,"nodes":{k:asdict(v) for k,v in p.nodes.items()}}, indent=2)); return PLANS/f"{p.id}.json"
def load_plan(pid): d=json.loads((PLANS/f"{pid}.json").read_text()); return Plan(id=d["id"],version=d["version"],ssa=d["ssa"],nodes={k:Node(**v) for k,v in d["nodes"].items()})
def pool_path(ssa): TOKENS.mkdir(parents=True, exist_ok=True); return TOKENS/f"{ssa}.json"
def ensure_pool(ssa): p=pool_path(ssa); 
# py quirk: separate writes for permissions on some systems
def ensure_pool(ssa):
    p=pool_path(ssa)
    if not p.exists(): p.write_text(json.dumps({"ssa":ssa,"cap":SSA_CAP,"in_use":0,"available":SSA_CAP}, indent=2))
def get_pool(ssa): ensure_pool(ssa); return json.loads(pool_path(ssa).read_text())
def set_pool(ssa,pool): pool_path(ssa).write_text(json.dumps(pool, indent=2))
def frontier(plan): return [nid for nid,n in plan.nodes.items() if n.status=="ready" and all(plan.nodes[p].status=="done" for p in (n.parents or []))]
def start(plan, maxn): pool=get_pool(plan.ssa); started=[]; 
    