# rmz Contribution Guide

This document describes how to contribute to rmz, a safe file deletion CLI tool written in Rust.

---

## 1. Project Overview & Contribution Flow

- **rmz** is a Rust CLI tool aiming to be a safe replacement for the traditional rm command.
- Bug fixes, new features, and documentation improvements are all welcome.
- We follow **Test-Driven Development (TDD)** as a principle.
- All work must be done on a **feature branch**; direct commits to the main branch are prohibited.

---

## 2. Development Environment Setup

- Rust 1.75 or later (latest stable recommended)
- cargo (Rust package manager)
- Recommended tools: clippy, rustfmt, cargo-audit

### Setup Steps
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/your-username/rmz.git
cd rmz

# Build dependencies and the project
cargo build

# Run tests
cargo test
```

---

## 3. Branch, Commit, and PR Rules

- Use descriptive branch names like **feature/feature-name** or **fix/bug-description**
- Always use `git worktree` to isolate your work
- Write commit messages in **English** and keep them concise (e.g., `Fix bug in trash feature`)
- When creating a PR, clearly state the purpose, changes, and test details
- Direct pushes to the main branch are not allowed

---

## 4. Testing & TDD Workflow

- All new features and fixes should follow **Test-Driven Development (TDD)**
- Write tests in the `tests/` directory or as `mod tests` in each module
- Run tests with: `cargo test`
- Do not merge to main until all tests pass

---

## 5. Documentation Update Rules

- Write documentation in Markdown under the `docs/` directory
- User documentation: place in `docs/`; developer documentation: place in `docs/dev/`
- Use lowercase and hyphen-separated filenames (e.g., architecture.md)
- Always update relevant docs when making significant changes or adding features
- Update README.md as needed

---

## 6. Code Style & Naming Conventions

- Follow Rust standard style (use rustfmt)
- Modules, functions, variables: snake_case
- Structs, enums: PascalCase
- Constants: SCREAMING_SNAKE_CASE
- File names: snake_case.rs
- Write comments and documentation in **English**

---

## 7. Security & Quality Management

Before submitting code, always run:
  - `cargo fmt` (formatting)
  - `cargo clippy` (lint)
  - `cargo test` (tests)
  - `cargo audit` (dependency vulnerability check)
- When adding dependencies, check licenses and security

---

## 8. FAQ & Troubleshooting

### Q. Tests fail or build does not work
- Check your Rust version (`rustc --version`)
- Try: `cargo clean && cargo build`

### Q. Commit is rejected
- Direct commits to main are not allowed. Always work on a feature branch.

### Q. What are the documentation naming and placement rules?
- User docs: `docs/`, developer docs: `docs/dev/`, filenames: lowercase with hyphens

### Q. Other questions
- Please open an Issue or start a Discussion.

---

We look forward to your contributions!