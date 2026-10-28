---
name: zed:guardrail
description: Inspect and enforce project guardrail rules, protected paths (e.g. common-grpc), and team policies.
---

# Project Guardrail Rules & Enforcement

When this skill is activated, you are inspecting and enforcing the project's guardrails to protect critical infrastructure and shared libraries.

## Workflow Instructions:
1. **Load Active Policies**:
   - Check `.zed/guardrails.yaml` or `.guardrails.yaml` in the repository root.
   - If connected to Cloud RAG, inspect fetched team policies.
2. **Verify Protected Boundaries**:
   - Identify denied paths (e.g. `common-grpc/**`, `proto/generated/**`, release workflows).
   - If the task involves modifying protected boundaries, inform the developer of the exact policy and alternative workflow (e.g. update proto definitions in service schemas, release protos via CI, never edit generated files directly).
3. **Audit Current Git State**:
   - Check `git status` and `git diff` against guardrail rules to ensure no forbidden files have been touched.
