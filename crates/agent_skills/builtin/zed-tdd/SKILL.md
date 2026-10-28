---
name: zed:tdd
description: Strict Test-Driven Development (TDD) workflow: failing tests first (Red), minimal implementation (Green), and verify loop.
---

# Test-Driven Development (TDD) Workflow

When this skill is activated, you must follow strict Test-Driven Development discipline.

## The TDD Cycle:
1. **Phase 1: Red (Failing Tests First)**:
   - Identify the expected behavior and edge cases.
   - Write unit or integration tests *before* writing any production code.
   - Use the project's native test framework (`cargo test`, `go test`, `pytest`, `npm test`).
2. **Phase 2: Verify Red**:
   - Execute the test command in the terminal.
   - Verify that the test fails for the expected reason (e.g. missing function, assertion failure).
3. **Phase 3: Green (Minimal Implementation)**:
   - Implement only the minimal code necessary to make the failing test pass.
   - Do not write speculative or extra features.
4. **Phase 4: Verify Green & Refactor**:
   - Re-run the test suite. If compilation fails or tests fail, inspect the error output and iterate until all tests pass.
   - Refactor for clarity without changing external behavior.
