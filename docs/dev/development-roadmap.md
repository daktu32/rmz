# Development Roadmap

This roadmap outlines the major phases, implementation plan, and progress for the rmz Rust CLI project.

---

## Vision
- Deliver a modern, safe, and intuitive replacement for the `rm` command
- Prevent accidental data loss and provide robust file management
- Cross-platform support (Linux, macOS, Windows)

---

## Phases & Milestones

### Phase 1: Foundation & Architecture
- TDD workflow established
- Domain models and layered architecture implemented
- CI/CD pipeline set up

### Phase 2: Core Infrastructure
- Metadata and trash storage implemented (JSON)
- Protected path management
- Cross-platform path handling

### Phase 3: Core Commands
- `delete`, `restore`, `list`, `status` commands implemented
- Comprehensive test suite
- User-friendly output and error handling

### Phase 4: Quality Assurance
- CI/CD with multi-platform testing
- Code quality checks (`clippy`, `fmt`)
- Security audit integration

### Phase 5: Management & Advanced Features
- `purge`, `log`, `protect`, `config`, `doctor`, `completions` commands
- Interactive restore (fzf integration)
- Operation logging and advanced filtering

### Phase 6: Distribution & Adoption
- Windows support and testing
- Package manager integration (Homebrew, Cargo, etc.)
- Binary distribution and installation scripts
- Documentation for users and developers

---

## Implementation Plan

- All work is done on feature branches using `git worktree`
- TDD: Write tests before implementation
- Update documentation and progress after each phase
- Use English for all code, comments, and documentation
- Regular code review and CI checks

---

## Progress & Next Steps

### Completed
- Foundation, architecture, and core infrastructure
- Core commands and test suite
- CI/CD and quality assurance

### In Progress / Next
- Implement `purge`, `log`, `protect`, `config`, `doctor`, `completions` commands
- Add interactive restore (fzf integration)
- Expand operation logging and advanced filtering
- Windows support and package manager integration
- Update documentation and user guides

---

## Metrics
- **Test Coverage**: >90%
- **Clippy Warnings**: 0
- **Security Vulnerabilities**: 0
- **Binary Size**: ~5MB (release build)

---

## Review & Updates
- Roadmap is reviewed after each major phase
- See `docs/dev/architecture.md` for system details
- See `docs/dev/test-strategy.md` for testing details