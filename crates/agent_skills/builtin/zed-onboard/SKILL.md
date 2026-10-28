---
name: zed:onboard
description: Scan codebase architecture, extract canonical patterns (controllers, models, tests), and ingest into Team RAG.
---

# Codebase Onboarding & RAG Ingestion Pipeline

When this skill is activated, you are performing an autonomous repository onboarding analysis. Your goal is to extract real implementation patterns from this codebase and ingest them into the Team RAG service to minimize hallucination.

## Workflow Instructions:
1. **Repository Discovery**:
   - Inspect the top-level directory structure, configuration files (`Cargo.toml`, `package.json`, `go.mod`, etc.).
   - Identify key layers:
     - **Controllers / Route Handlers**: Where HTTP/gRPC requests enter the application.
     - **Service / Business Logic**: Where application business logic resides.
     - **Models / Schemas / Entities**: Data structures, ORM/DB entities, validation schemas.
     - **Test Suites**: How unit, integration, and end-to-end tests are written.
2. **Extract Canonical Patterns**:
   - For each layer, extract 2-3 clean, representative code snippets with file path references.
   - Note critical project conventions (e.g. error handling, logging, middleware).
3. **Ingest to Team RAG**:
   - Format documents with categories (`controller`, `models`, `architecture`, `testing`, `grpc`).
   - If `zed-rag-service` is available at configured URL (default: `http://localhost:8080`), submit the payload via `POST /api/v1/workspaces/{workspace_id}/onboard`.
4. **Summary Report**:
   - Provide the developer with a clear architecture summary in the chat, listing the discovered patterns and confirming indexing status.
