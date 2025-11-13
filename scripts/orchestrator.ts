#!/usr/bin/env -S node --enable-source-maps
import fs from 'node:fs';
import { spawnSync } from 'node:child_process';
import crypto from 'node:crypto';

interface TaskSpec { id:string; title:string; scope_paths:string[]; forbidden_paths:string[]; budgets:{max_files:number; max_additions:number; allow_deletions:boolean}; acceptance:string[]; artifacts:string[]; evidence_ledger:string; agents:string[] }

function sh(cmd:string){
  const r=spawnSync(cmd,{shell:true,stdio:'inherit'});
  if(r.status!==0) throw new Error(`cmd failed: ${cmd}`);
}

function hashFile(p:string){
  const b=fs.readFileSync(p); return crypto.createHash('sha256').update(b).digest('hex');
}

function main(){
  const id=process.argv[2]||''; if(!id) throw new Error('usage: scripts/orchestrator.ts <task-id or path>');
  const specPath=id.endsWith('.json')?id:`workflow/tasks/${id}.json`;
  const spec=JSON.parse(fs.readFileSync(specPath,'utf-8')) as TaskSpec;
  for(const cmd of spec.acceptance){ sh(cmd); }
  const artifacts:string[]=[]; const hashes:Record<string,string>={};
  for(const glob of spec.artifacts){
    if(glob.includes('*')) continue; // minimal: only exact paths for now
    if(fs.existsSync(glob)){ artifacts.push(glob); hashes[glob]=hashFile(glob); }
  }
  const ledger={ task_id: spec.id, status: 'PASS', artifacts, checks: spec.acceptance, hashes };
  fs.mkdirSync(require('node:path').dirname(spec.evidence_ledger),{recursive:true});
  fs.writeFileSync(spec.evidence_ledger, JSON.stringify(ledger,null,2));
  console.log(`Ledger written: ${spec.evidence_ledger}`);
}
main();
