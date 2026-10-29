---
name: zed:usage
description: Read token usage metrics and subscription quota utilization percentage for Claude, Codex, Copilot, and Antigravity.
---

# Subscription Quota & Token Usage Inspector

When this skill is activated, you are inspecting and reporting subscription quota utilization and token consumption metrics.

## Workflow Instructions:
1. **Read Active Subscription Quota & Accounts**:
   - Inspect local subscriptions from the machine:
     - **Claude (Anthropic)**: 5-hour session & 7-day weekly quota percentages, account email, and plan tier (`~/.claude.json`).
     - **OpenAI Codex**: Account ID, email, auth mode, and active subscription status (`~/.codex/auth.json`).
     - **GitHub Copilot**: Username, app ID, scopes, and active subscription status (`~/.config/github-copilot/apps.json`).
     - **Google Antigravity**: Account email, OAuth session status, and platform tier (`~/.gemini/oauth_creds.json`).
2. **Report Token Counts Per Provider**:
   - Report cumulative session token counts:
     - **Claude (Anthropic)**: Input, Output, and Cache Read/Write tokens.
     - **Codex (OpenAI)**: Input, Output tokens.
     - **GitHub Copilot**: Input, Output tokens.
     - **Antigravity (Google AI)**: Input, Output tokens.
   - Calculate percentage of active context window consumed.
3. **Present Clean Summary**:
   - Format the results into a high-signal markdown report with percentage badges and countdowns to quota resets.
