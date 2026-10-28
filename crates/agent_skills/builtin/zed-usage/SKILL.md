---
name: zed:usage
description: Read token usage metrics and subscription quota utilization percentage for Claude and Codex.
---

# Subscription Quota & Token Usage Inspector

When this skill is activated, you are inspecting and reporting subscription quota utilization and token consumption metrics.

## Workflow Instructions:
1. **Read Active Subscription Quota**:
   - Inspect the Claude subscription utilization from the local environment (`~/.claude.json`):
     - **5-Hour Session Quota**: Utilization percentage and next reset timestamp.
     - **7-Day Weekly Quota**: Utilization percentage and next reset timestamp.
     - **Account & Tier**: Display plan tier (e.g. Pro, Team) and rate-limit tier.
2. **Report Token Counts Per Provider**:
   - Report cumulative session token counts:
     - **Claude (Anthropic)**: Input, Output, and Cache Read/Write tokens.
     - **Codex / Copilot (OpenAI)**: Input, Output tokens.
   - Calculate percentage of active context window consumed.
3. **Present Clean Summary**:
   - Format the results into a high-signal markdown report with percentage badges and countdowns to quota resets.
