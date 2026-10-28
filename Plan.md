# Implementation Plan: Zed Harness Agent & Cloud RAG Platform

**Version:** 1.0.0  
**Target Branch:** `master`  
**Reference Document:** [`PRD.md`](file:///Users/dimasmaulana/Documents/Research/zed/PRD.md)

---

## Architecture Milestone Overview

```
Phase 1: Guardrail Engine (Client-Side Interceptor + Local Config)
   │
   ▼
Phase 2: Cloud RAG & Guardrail Backend (Separate Repository Blueprint & Docker CI/CD)
   │
   ▼
Phase 3: Autonomous Onboarding & Knowledge Ingestion (`onboard` pipeline)
   │
   ▼
Phase 4: Subscription Harvester & TDD/SDD Orchestration
   │
   ▼
Phase 5: Agent-First Harness UI (Kiro / Cursor Experience in GPUI)
```

---

## Phase 1: Local & Cloud Guardrail Enforcement

### Objective
Prevent the AI Agent from modifying forbidden paths (e.g. `common-grpc/**`) or running unauthorized shell commands.

### Key Work Items
1. **Define Schema (`crates/agent_settings` or `crates/agent`)**:
   - Create `GuardrailConfig` struct supporting `deny_paths`, `path_rules`, `protected_branches`, and `allowed_commands`.
   - Implement YAML/JSON parsing for `.zed/guardrails.yaml` and `.rules`.
2. **Implement Guardrail Interceptor (`crates/agent/src/guardrails.rs`)**:
   - Function: `check_tool_call(tool_name, args, config) -> Result<(), GuardrailViolation>`
   - Intercept file editing tools: `write_file`, `edit_file`, `create_directory`.
   - Intercept terminal execution: `run_command` / bash execution.
3. **Hook into Agent Execution Pipeline (`crates/agent/src/tools.rs`)**:
   - Ensure violated rules inject clear error explanations back into the model prompt context (e.g., *"Blocked: common-grpc is protected. Update proto service definitions instead."*).
4. **Unit & Integration Tests**:
   - Test pattern matching with glob syntax (`common-grpc/**`, `*.proto`).
   - Validate error messaging and prompt feedback loop.

---

## Phase 2: Cloud RAG & Team Guardrail Backend (Separate Repository)

### Objective
Create a dedicated microservice repository (`zed-rag-engine` / `cloud-rag-service`) with independent Docker CI/CD and SemVer tagging.

### Architecture & Tech Stack
* **Language & Framework**: Python (FastAPI / Pydantic) or Rust (Axum / Tokio).
* **Vector Engine**: Qdrant or pgvector.
* **Embeddings**: FastEmbed (local/lightweight) or OpenAI/Voyage embeddings.
* **Storage**: PostgreSQL / SQLite for metadata and team guardrail policies.

### Core API Endpoints
* `POST /api/v1/workspaces/{id}/onboard`: Receives codebase documentation and architectural rules for embedding and indexing.
* `POST /api/v1/workspaces/{id}/query`: Hybrid search (BM25 + Semantic) returning relevant code conventions and examples.
* `GET /api/v1/workspaces/{id}/guardrails`: Returns team-wide guardrail rules to the Zed client on startup.
* `POST /api/v1/workspaces/{id}/guardrails`: Team lead endpoint to update shared repository rules.

### Docker & CI/CD Pipeline
* `Dockerfile`: Multi-stage, lightweight production container.
* `.github/workflows/docker-publish.yml`:
  - Triggers on tag push (`v*.*.*`).
  - Builds multi-arch (`linux/amd64`, `linux/arm64`) Docker images.
  - Publishes to GitHub Container Registry (`ghcr.io`) or Docker Hub.

---

## Phase 3: Autonomous "Onboard" Pipeline in Zed

### Objective
When the user types `onboard`, the Agent analyzes the active repository, extracts real code patterns, and pushes them into the Cloud RAG store.

### Key Work Items
1. **Command Registration (`crates/agent`)**:
   - Register the `/onboard` slash command or prompt interceptor.
2. **Codebase Pattern Extractor (`crates/agent/src/onboarding.rs`)**:
   - Scans directory tree, dependencies (`Cargo.toml`, `package.json`, `go.mod`, etc.).
   - Locates controllers, services, repositories, schemas, and test suites.
   - Extracts 3-5 idiomatic code examples per layer.
3. **Structured Synthesis**:
   - Prompts the LLM to format the findings into a standard reference guide:
     - Architecture overview.
     - Controller patterns & route handlers.
     - Error handling conventions.
     - Data validation & serialization.
     - Testing conventions.
4. **RAG Client Integration (`crates/cloud_api_client` or `crates/agent_servers`)**:
   - Sends the synthesized knowledge chunks to `POST /api/v1/workspaces/{id}/onboard`.
   - Confirms indexing completion to the user in chat.

---

## Phase 4: Subscription Harvester & TDD / SDD Orchestration

### Objective
Eliminate API key hurdles by reusing existing authenticated subscriptions, and enforce strict Spec-Driven & Test-Driven development cycles.

### Key Work Items
1. **Subscription Harvester (`crates/credentials_provider`)**:
   - **Claude CLI Reader**: Checks `~/.claude.json` and macOS Keychain for valid session tokens.
   - **Copilot / Codex Reader**: Reads GitHub Copilot tokens from `~/.config/github-copilot/hosts.json` and system credential store.
   - Register harvested accounts into [`crates/language_model/src/registry.rs`](file:///Users/dimasmaulana/Documents/Research/zed/crates/language_model/src/registry.rs).
2. **SDD Prompt & State Machine (`crates/agent/src/sdd.rs`)**:
   - Forces phase transitions:
     - **Phase 1: Specification**: Generate `spec.md` with acceptance criteria. Pause for user confirmation.
     - **Phase 2: Test Suite Generation**: Write tests in the project's native test framework (`cargo test`, `pytest`, `jest`).
     - **Phase 3: Implementation**: Implement business logic.
     - **Phase 4: Auto-Verification Loop**: Execute tests via Zed terminal runner, re-prompting on failures until 100% green.

---

## Phase 5: Harness Agent UI (Agent-First / Kiro Layout)

### Objective
Make the Agent Chat the primary center window in Zed, with code editors and diff reviews appearing as contextual drawers.

### Key Work Items
1. **Workspace Center Re-architecture (`crates/workspace/src/workspace.rs`)**:
   - Add workspace mode: `AgentFirst` vs `EditorFirst`.
   - In `AgentFirst` mode, [`crates/agent_ui/src/agent_panel.rs`](file:///Users/dimasmaulana/Documents/Research/zed/crates/agent_ui/src/agent_panel.rs) occupies the central pane.
2. **Contextual Editor & Diff Overlays**:
   - When the agent edits files, automatically show a split or floating multi-buffer diff view ([`crates/git_ui`](file:///Users/dimasmaulana/Documents/Research/zed/crates/git_ui) & [`crates/multi_buffer`](file:///Users/dimasmaulana/Documents/Research/zed/crates/multi_buffer)).
   - Add inline review actions: `Accept All`, `Reject All`, `Review in Split Editor`.
3. **Git Tree & File Explorer Drawers**:
   - Keep file navigation and git history available as toggleable slide-out drawers.

---

## Immediate Next Steps (Sprint 1)

| Task | Target Files | Status |
| :--- | :--- | :--- |
| 1. Implement Local Guardrail Engine | `crates/agent/src/guardrails.rs`, `crates/agent/src/tools.rs` | Ready to start |
| 2. Create Standalone RAG Service Spec & Dockerfile template | Repo blueprint for `zed-rag-engine` | Ready to start |
| 3. Add `.zed/guardrails.yaml` sample to repo | `.zed/guardrails.example.yaml` | Ready to start |
