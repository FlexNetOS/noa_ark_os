# CSV Schema v1.0 (Mandatory Columns)

Use these exact headers (lowercase, underscores). Extra columns are allowed but ignored by the normalizer.

```
agent_name, role, layer, scope, tools, inputs, outputs, guardrails, escalation_to, stack
```

**Allowed `layer` values (enum):**
- `cecca` | `board` | `executive` | `stack_chief` | `specialist` | `micro`

**Examples:**
```
agent_name,role,layer,scope,tools,inputs,outputs,guardrails,escalation_to,stack
Chief Orchestrator,Chief Executive,cecca,global,cli|python,,artifact://plan,offline-first|audited,Board,Global
Policy Chair,Policy Lead,board,governance,cli,,artifact://policy,offline-first|no-external-saas,CECCA,Global
Ops Commander,Operations Exec,executive,operations,bash|python,artifact://plan,artifact://ops,offline-first,CECCA,Ops
Stack Alpha Chief,Stack Orchestrator,stack_chief,subject-alpha,python|local_tool_A,artifact://subject-alpha,artifact://subject-alpha/out,offline-first,Executive,Subject-Alpha
Spec A1,Data Ingestion,specialist,subject-alpha,python,artifact://subject-alpha,artifact://subject-alpha/ds,offline-first,Stack-Chief,Subject-Alpha
Micro A1-1,Parser,micro,subject-alpha,python,artifact://subject-alpha/ds,artifact://subject-alpha/ds/parsed,offline-first,Spec A1,Subject-Alpha
```
