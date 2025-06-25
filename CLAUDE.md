# CLAUDE.md

This document provides guidance for contributors and AI agents working on the rmz project—a safe file deletion CLI tool written in Rust.

---

## Project Overview

**rmz** is a modern, safe replacement for the traditional `rm` command. Instead of permanently deleting files, rmz moves them to a TrashZone, allowing for easy recovery and improved safety.

- **Language:** Rust
- **CLI Framework:** clap v4
- **Testing:** cargo test
- **Configuration:** TOML
- **Documentation:** Markdown in `docs/` and `docs/dev/`

---

## Architecture

- **CLI Layer:** Command parsing and user interaction (clap)
- **Core Logic:** File operations, metadata management, history tracking
- **Storage:** TrashZone (`~/.rmz/trash/`), metadata as JSON
- **Configuration:** User settings, protected paths

Directory structure:
```
rmz/
├── src/            # Rust source code
├── tests/          # Integration tests
├── docs/           # User and developer documentation
│   └── dev/        # Developer docs
├── scripts/        # Utility scripts
├── Cargo.toml      # Project manifest
```

---

## Development Philosophy

- **Test-Driven Development (TDD):** Write tests before implementation
- **Clean Code:** Maintainable, readable, and well-documented
- **Security First:** Follow best practices from the start
- **Performance:** Optimize for speed and efficiency

---

## Key Features

- Safe file deletion (`rmz delete`): Moves files to TrashZone
- File restore (`rmz restore`): Recover deleted files
- List deleted files (`rmz list`): View trash contents
- Permanent delete (`rmz purge`): Remove files from TrashZone
- Protected paths: Prevent deletion of critical files

---

## Contribution & Workflow Rules

- All work must be done on a feature branch (never commit directly to main)
- Use `git worktree` for branch isolation
- Follow TDD: write tests first, then implement
- Update tests in `tests/` or as `mod tests` in modules
- Update documentation in `docs/` and `docs/dev/` as needed
- Use English for all code comments, documentation, and commit messages
- Run `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo audit` before submitting code
- Always update `docs/dev/progress.md` and `docs/dev/development-roadmap.md` after significant changes

---

## Coding Standards

- Modules, functions, variables: `snake_case`
- Structs, enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- File names: `snake_case.rs`
- Documentation: Markdown, English, lowercase and hyphen-separated filenames

---

## Checklist Before Submitting

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] Progress files are updated
- [ ] Commit messages are clear and in English
- [ ] Pull Request includes a summary of changes and test results

---

## FAQ & Troubleshooting

- **Tests fail or build does not work:**
  - Check Rust version (`rustc --version`)
  - Run `cargo clean && cargo build`
- **Commit is rejected:**
  - Never commit directly to main; always use a feature branch
- **Documentation rules:**
  - User docs: `docs/`, developer docs: `docs/dev/`, filenames: lowercase with hyphens
- **Other questions:**
  - Open an Issue or Discussion on GitHub

---

Thank you for contributing to rmz!
