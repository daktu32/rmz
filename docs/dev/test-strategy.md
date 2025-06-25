# Test Strategy

This document describes the test strategy for the rmz Rust CLI project, following TDD and modern OSS best practices.

---

## Overview

rmz is a critical tool for safe file deletion and recovery. Comprehensive testing is required to ensure reliability and safety. The project follows Test-Driven Development (TDD): tests are written before implementation.

---

## Test Pyramid

```
        /\
       /  \
      /E2E \    (few: actual CLI execution)
     /______\
    /Integration\  (some: command integration)
   /__________\
  /  Unit Tests \  (many: individual functions)
 /______________\
```

---

## Test Levels

### 1. Unit Tests
- Target: individual functions, methods, and data structures
- Framework: Rust built-in `#[test]`
- Mocking: `mockall`
- Coverage goal: 80%+

### 2. Integration Tests
- Target: command interactions, file I/O
- Framework: `tests/` directory
- Environment: `tempfile` for isolated directories
- Run: `cargo test --test integration_tests`

### 3. End-to-End (E2E) Tests
- Target: actual CLI execution
- Framework: `assert_cmd`
- Run: `cargo test --test integration_tests` (with real binaries)

---

## TDD Workflow

1. **Red**: Write a failing test for the new feature or bug fix
2. **Green**: Implement the minimal code to pass the test
3. **Refactor**: Improve the code while keeping tests passing

---

## Quality Standards
- All code must be covered by tests
- No clippy warnings allowed
- All tests must pass in CI/CD
- Test data and fixtures are managed in Rust (see `tests/`)
- All code, comments, and documentation are in English

---

## References
- See `docs/dev/architecture.md` for system structure
- See `docs/dev/development-roadmap.md` for milestones