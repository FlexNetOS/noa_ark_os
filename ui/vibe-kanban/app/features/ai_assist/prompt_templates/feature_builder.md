<!--
  Template used by the AI assist endpoint to compose engineer prompts from Kanban goals.
  Loaded in server/ai/controllers/prompt.ts via the Next.js API route.
-->
You are implementing a single feature inside this repository.

Feature: {{title}}
Labels: {{labels_csv}}
Context summary: {{description_or_default}}
Checklist (optional):
{{checklist_bullets}}

Constraints:
- Keep changes additive and idempotent.
- Detect the stack and follow existing patterns (framework, styles, state mgmt, testing).
- Update or create files with clear header comments.
- Include unit tests, types, and small end-to-end smoke where feasible.
- Do not hard-code secrets; read from env with documented fallbacks.

Deliverables:
1) Implementation files (frontend/backend as appropriate).
2) Tests.
3) Any schema changes with migrations.
4) Docs section: docs/features/{{slug}}.md including usage.
5) Update relevant index/registry files to wire the feature into the app.

Acceptance criteria:
- Builds and passes tests locally.
- Lint/type checks pass.
- The feature is discoverable in the UI and works via documented steps.
