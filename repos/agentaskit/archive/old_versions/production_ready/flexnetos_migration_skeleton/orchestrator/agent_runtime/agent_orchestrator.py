#!/usr/bin/env python3
import argparse, json, time, uuid, sys
from pathlib import Path
from dataclasses import dataclass, asdict

ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / "orchestrator" / "state"
LOG = STATE / "event_log.jsonl"
PLANS = STATE / "plans"
TOKENS = STATE / "tokens"
SSA_CAP = 50  # cap per SSA

def now(): return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())

def append_event(ev):
    LOG.parent.mkdir(parents=True, exist_ok=True)
    with open(LOG, "a", encoding="utf-8") as f: f.write(json.dumps(ev)+"\n")

@dataclass
class Node:
    id: str
    spec: dict
    status: str = "ready"  # ready|running|paused|parked|done|failed|cancelled
    budget: dict = None
    parents: list = None
    children: list = None
    attempts: int = 0

@dataclass
class Plan:
    id: str
    version: int
    nodes: dict
    ssa: str

def save_plan(plan: Plan):
    PLANS.mkdir(parents=True, exist_ok=True)
    p = PLANS / f"{plan.id}.json"
    with open(p,"w",encoding="utf-8") as f: json.dump({"id":plan.id,"version":plan.version,"ssa":plan.ssa,"nodes":{k:asdict(v) for k,v in plan.nodes.items()}}, f, indent=2)
    return p

def load_plan(pid:str)->Plan:
    data = json.loads((PLANS/f"{pid}.json").read_text())
    nodes = {k: Node(**v) for k,v in data["nodes"].items()}
    return Plan(id=data["id"], version=data["version"], nodes=nodes, ssa=data["ssa"])

def pool_path(ssa:str)->Path:
    TOKENS.mkdir(parents=True, exist_ok=True)
    return TOKENS/f"{ssa}.json"

def ensure_pool(ssa:str):
    p = pool_path(ssa)
    if not p.exists():
        p.write_text(json.dumps({"ssa":ssa,"cap":SSA_CAP,"in_use":0,"available":SSA_CAP}, indent=2))

def get_pool(ssa:str)->dict:
    ensure_pool(ssa); return json.loads(pool_path(ssa).read_text())

def set_pool(ssa:str, pool:dict):
    pool_path(ssa).write_text(json.dumps(pool, indent=2))

def frontier(plan: Plan):
    out = []
    for nid, n in plan.nodes.items():
        if n.status != "ready": continue
        parents = n.parents or []
        if all(plan.nodes[p].status == "done" for p in parents): out.append(nid)
    # Priorities: shallower depth first, then aged attempts, then fair
    out.sort()
    return out

def start(plan: Plan, max_to_start:int):
    pool = get_pool(plan.ssa)
    started = []
    for nid in frontier(plan):
        if len(started) >= max_to_start or pool["available"] <= 0: break
        n = plan.nodes[nid]
        n.status = "running"; n.attempts += 1
        pool["available"] -= 1; pool["in_use"] += 1
        started.append(nid)
        append_event({"ts":now(),"type":"start","plan":plan.id,"node":nid})
    set_pool(plan.ssa, pool)
    return started

def submit_pop(plan: Plan, node_id:str, ok:bool, checkpoint_hash:str, side_effect_log_hash:str):
    pool = get_pool(plan.ssa)
    n = plan.nodes[node_id]
    if n.status != "running": raise SystemExit(f"node {node_id} not running")
    n.status = "done" if ok else "parked"
    pool["in_use"] = max(0, pool["in_use"]-1)
    pool["available"] = min(SSA_CAP, pool["available"]+1)
    set_pool(plan.ssa, pool)
    append_event({"ts":now(),"type":"pop","plan":plan.id,"node":node_id,"ok":ok,
                  "checkpoint":checkpoint_hash,"side_effects":side_effect_log_hash})

def cmd_init(args):
    pid = args.plan or str(uuid.uuid4())[:8]; ssa = args.ssa or "ssa-1"
    nodes = {
        "A": Node(id="A", spec={"tool":"featureize"}, parents=[], children=["B"]),
        "B": Node(id="B", spec={"tool":"classify"}, parents=["A"], children=["C"]),
        "C": Node(id="C", spec={"tool":"store"}, parents=["B"], children=[])
    }
    plan = Plan(id=pid, version=1, nodes=nodes, ssa=ssa)
    save_plan(plan); ensure_pool(ssa); append_event({"ts":now(),"type":"plan-create","plan":pid,"ssa":ssa})
    print(f"[init-plan] {pid} in {PLANS}")

def cmd_tick(args):
    plan = load_plan(args.plan); started = start(plan, args.max_start); save_plan(plan)
    print(json.dumps({"started":started,"pool":get_pool(plan.ssa),"frontier":frontier(plan)}, indent=2))

def cmd_pop(args):
    plan = load_plan(args.plan); submit_pop(plan, args.node, args.ok, args.checkpoint_hash, args.side_effect_log_hash); save_plan(plan)
    print(json.dumps({"node":args.node,"status":plan.nodes[args.node].status,"pool":get_pool(plan.ssa)}, indent=2))

def cmd_status(args):
    plan = load_plan(args.plan); pool = get_pool(plan.ssa)
    print(json.dumps({"pool":pool,"status":{k:v.status for k,v in plan.nodes.items()},"frontier":frontier(plan)}, indent=2))

def cmd_demo(args):
    # deterministic demo: A -> B -> C completes using PT pool mechanics
    cmd_init(type("o",(),{"plan":"demo","ssa":"ssa-demo"})())
    cmd_tick(type("o",(),{"plan":"demo","max_start":50})())
    cmd_pop(type("o",(),{"plan":"demo","node":"A","ok":True,"checkpoint_hash":"ckA","side_effect_log_hash":"seA"})())
    cmd_tick(type("o",(),{"plan":"demo","max_start":50})())
    cmd_pop(type("o",(),{"plan":"demo","node":"B","ok":True,"checkpoint_hash":"ckB","side_effect_log_hash":"seB"})())
    cmd_tick(type("o",(),{"plan":"demo","max_start":50})())
    cmd_pop(type("o",(),{"plan":"demo","node":"C","ok":True,"checkpoint_hash":"ckC","side_effect_log_hash":"seC"})())
    cmd_status(type("o",(),{"plan":"demo"})())

def main():
    import argparse
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers()

    p0 = sub.add_parser("init-plan"); p0.add_argument("--plan"); p0.add_argument("--ssa"); p0.set_defaults(func=cmd_init)
    p1 = sub.add_parser("tick"); p1.add_argument("--plan", required=True); p1.add_argument("--max-start", type=int, default=50); p1.set_defaults(func=cmd_tick)
    p2 = sub.add_parser("submit-pop"); p2.add_argument("--plan", required=True); p2.add_argument("--node", required=True); p2.add_argument("--ok", action="store_true"); p2.add_argument("--checkpoint_hash", required=True); p2.add_argument("--side_effect_log_hash", required=True); p2.set_defaults(func=cmd_pop)
    p3 = sub.add_parser("status"); p3.add_argument("--plan", required=True); p3.set_defaults(func=cmd_status)
    p4 = sub.add_parser("--demo", add_help=False); p4.set_defaults(func=cmd_demo)

    args = ap.parse_args()
    if not hasattr(args, "func"): ap.print_help(); sys.exit(2)
    args.func(args)

if __name__ == "__main__":
    main()
