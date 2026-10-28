# Product Requirements Document (PRD): Zed Harness Agent & Cloud RAG Platform

**Version:** 1.0.0-draft  
**Target:** Zed Harness Agent (Client) + Cloud Team RAG & Guardrail Service (Backend)  
**Branch:** `master`

---

## 1. Executive Summary

The **Zed Harness Agent** transforms Zed from a traditional code editor into an **Agent-First IDE** (similar to Kiro and Cursor), where the primary user interface is an autonomous AI programming harness. 

To ensure production-grade accuracy and prevent hallucinations, the system is backed by:
1. **Cloud RAG & Guardrail Platform** (decoupled microservice with dedicated Docker CI/CD and release cycles).
2. **Onboarding Knowledge Extraction**: An autonomous pipeline triggered by `onboard` that analyzes the target repository, synthesizes canonical code patterns (e.g., how to write controllers, error handling, DTOs), and indexes them into the RAG vector store.
3. **Spec-Driven (SDD) & Test-Driven (TDD) Execution Loops**: Enforces strict phases (Spec -> Failing Test -> Minimal Code -> Verification) powered by local Claude/Codex/Copilot subscriptions already authenticated on the developer machine.
4. **Cloud & Local Guardrails**: Policy engine enforcing file/directory write restrictions (e.g., forbidding direct edits to `common-grpc` and enforcing proto releases).

---

## 2. Architecture & Ecosystem Overview

```
┌────────────────────────────────────────────────────────────────────────┐
│                        DEVELOPER MACHINE                               │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                ZED HARNESS AGENT (Client / Rust)                 │  │
│  │                                                                  │  │
│  │  ┌─────────────────────────┐     ┌────────────────────────────┐  │  │
│  │  │   Agent-First UI        │     │  TDD / SDD Orchestrator    │  │  │
│  │  │  (Kiro-style Main Chat) │     │  (Spec -> Test -> Verify)  │  │  │
│  │  └───────────┬─────────────┘     └─────────────┬──────────────┘  │  │
│  │              │                                 │                 │  │
│  │  ┌───────────▼─────────────┐     ┌─────────────▼──────────────┐  │  │
│  │  │ Local Subscription Mgr  │     │  Guardrail Interceptor     │  │  │
│  │  │ (Claude/Codex/Copilot)  │     │  (Policy Enforcement)      │  │  │
│  │  └─────────────────────────┘     └─────────────┬──────────────┘  │  │
│  └────────────────────────────────────────────────┼─────────────────┘  │
└───────────────────────────────────────────────────┼────────────────────┘
                                                    │ HTTPS / gRPC
                                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                  CLOUD TEAM PLATFORM (Separate Repo)                   │
│                                                                        │
│  ┌───────────────────────────┐        ┌─────────────────────────────┐  │
│  │  Team Guardrail Registry  │        │   RAG Vector & BM25 Engine  │  │
│  │  (Team Policies & Rules)  │        │   (Qdrant / pgvector)       │  │
│  └───────────────────────────┘        └─────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Onboard Ingestion Service (Codebase Patterns & Few-Shot Index)  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│  Independent Docker CI/CD & SemVer Release Tagging                      │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Core Capabilities & Requirements

### 3.1. Pillar 1: Cloud RAG & Onboard Knowledge Extraction

#### Functional Requirements
* **Separate Repository Strategy**: The RAG service lives in its own repository (`zed-rag-engine` or `cloud-rag-service`) to prevent release tag collisions with the Zed editor binary releases.
* **Autonomous "Onboard" Command**:
  - When the developer executes `onboard` in the chat, the agent triggers a repo-wide structural analysis.
  - Generates standardized architectural guides tailored to the codebase:
    - *Controller / Route Conventions* (with concrete code samples from the project).
    - *Service & Business Logic Patterns*.
    - *Data Models & Validation Rules*.
    - *Testing Standards & Fixture Setup*.
  - Converts synthesized guides into structured vector embeddings + metadata.
  - Ingests into the Cloud RAG vector store under the repository's workspace ID.
* **Query & Context Augmentation**:
  - Before the agent generates new code, it queries RAG for matching reference implementations.
  - Automatically injects local codebase conventions into the prompt context to eliminate hallucinations.
* **CI/CD & Deployment**:
  - Fully containerized via `Dockerfile` and `docker-compose.yml`.
  - Automated GitHub Actions building multi-arch Docker images on release tags.

---

### 3.2. Pillar 2: TDD & SDD Engine with Subscription Harvesting

#### Functional Requirements
* **Existing Subscription Harvester**:
  - Automatically detects and harvests authenticated tokens from the developer's workstation:
    - **Claude Code CLI**: `~/.claude.json` / macOS Keychain.
    - **GitHub Copilot / Codex**: `~/.config/github-copilot/hosts.json` / System Keychain.
    - **VSCode OAuth Sessions**: VSCode secure storage tokens.
  - Enables zero-friction usage without requiring developers to provision separate paid API keys.
* **Spec-Driven Development (SDD) Workflow**:
  - **Step 1 (Spec)**: Agent writes or updates a `spec.md` with explicit functional requirements, API contracts, and edge cases.
  - User reviews and confirms the spec before code generation begins.
* **Test-Driven Development (TDD) Workflow**:
  - **Step 2 (Failing Tests)**: Agent writes unit / integration test files before modifying application logic.
  - **Step 3 (Run Tests)**: Executes tests via Zed's terminal/task runner to assert initial failure (Red).
  - **Step 4 (Implementation)**: Agent produces minimal implementation code.
  - **Step 5 (Verify & Loop)**: Re-runs tests (Green). If compiler errors or test failures occur, the output is fed back to the agent for self-healing.

---

### 3.3. Pillar 3: Dual-Layer Guardrail System (Cloud + Local)

#### Functional Requirements
* **Team-Wide Cloud Policies**:
  - The Cloud RAG service exposes a Guardrail API endpoint: `GET /api/v1/workspaces/{id}/guardrails`.
  - Team administrators can configure centralized rules enforced across all team members.
* **Local Fallback Policies**:
  - Supports repository-level `.zed/guardrails.yaml` or `.rules`.
* **Policy Syntax & Constraints**:
  ```yaml
  version: "1.0"
  guardrails:
    deny_paths:
      - "common-grpc/**"
      - "proto/generated/**"
      - ".github/workflows/**"
    path_rules:
      - pattern: "common-grpc/**"
        action: block
        reason: "Direct edits to common-grpc are strictly forbidden. You must edit service proto files and run the proto release task."
    protected_branches:
      - "main"
      - "master"
    allowed_commands:
      - "cargo test"
      - "cargo check"
      - "npm test"
  ```
* **Enforcement Point**:
  - Intercepts all Agent Tool calls (`write_file`, `edit_file`, `run_command`).
  - If a violation occurs, the tool call is blocked immediately, returning the specific `reason` directly into the agent context so it can correct its approach.

---

### 3.4. Pillar 4: Harness Agent UI (Agent-First / Kiro Experience)

#### Functional Requirements
* **Center Stage Chat**:
  - The default center view in the workspace is the `AgentPanel` / `ConversationView`.
  - The developer interacts with the AI agent as the primary workflow driver.
* **Contextual Auxiliary Panels**:
  - Code editors, file trees, and git graphs open in contextual drawers or secondary split panes on-demand.
  - When the agent touches files, an interactive **Diff Review** tab appears with side-by-side or inline chunk acceptance (`Accept`, `Reject`, `Refine`).
* **Git Status & Review**:
  - One-click access to Git changes, visual tree, and commit review without leaving the agent conversation.

---

## 4. Success Metrics

1. **Zero Hallucination Rate on Code Patterns**: Generated code conforms to repository patterns (naming, error handling, layer architecture) in >95% of tasks.
2. **Guardrail Compliance**: 100% of unauthorized file writes to protected paths (`common-grpc`, etc.) are intercepted and blocked.
3. **Frictionless Auth**: Instant activation for users with active Claude or Copilot subscriptions without requiring manual API key configuration.
4. **TDD Loop Completion**: Automatically drives features from failing test to passing test without manual test execution by the user.

---

## 5. Non-Functional Requirements

* **Performance**: Sub-10ms guardrail check overhead on tool execution.
* **Platform Support**: macOS (Metal GPU acceleration), Linux, and Windows.
* **Security**: Zero credential leakage; harvested subscription tokens stay in memory and are never transmitted to external telemetry.
