---
name: zed:sdd
description: Spec-Driven Development (SDD) workflow: formal specification (spec.md), user approval, and acceptance-criteria verification.
---

# Spec-Driven Development (SDD) Workflow

When this skill is activated, you must follow strict Spec-Driven Development discipline.

## The SDD Cycle:
1. **Phase 1: Draft Specification (`spec.md`)**:
   - Write or update `spec.md` with:
     - **Goals & Non-Goals**: Clear problem statement and scope boundaries.
     - **Architecture & System Design**: Component interactions and data flow.
     - **API Contracts / Schemas**: Request, response, error formats, data structures.
     - **Acceptance Criteria**: Concrete, testable conditions for completion.
     - **Edge Cases & Failure Modes**: Network partitions, invalid inputs, error handling.
2. **Phase 2: Human Confirmation**:
   - Present the spec summary to the user.
   - Pause and request user review and approval before proceeding with any implementation.
3. **Phase 3: Derive Tests & Implementation**:
   - Translate each acceptance criterion into a test case.
   - Implement code satisfying the approved specification.
4. **Phase 4: Specification Verification**:
   - Verify every item in the acceptance criteria is fulfilled and tested.
