# Basic Development Guide

This guide describes the basic development setup and workflow for the rmz Rust CLI project.

---

## Prerequisites
- Rust toolchain (latest stable)
- Git
- Editor/IDE (VSCode, CLion, etc.)
- Unix-like environment (Linux, macOS, or WSL)

---

## Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/rmz.git
   cd rmz
   ```
2. Install Rust (if not already):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```
3. Run tests:
   ```bash
   cargo test
   ```
4. Build the CLI:
   ```bash
   cargo build --release
   ```

---

## Development Workflow
- Use feature branches and `git worktree` for all changes
- Follow TDD: write tests before implementation
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before committing
- Update documentation as needed
- All code, comments, and documentation must be in English

---

## References
- See `CONTRIBUTING.md` for detailed contribution guidelines
- See `docs/dev/architecture.md` for system structure