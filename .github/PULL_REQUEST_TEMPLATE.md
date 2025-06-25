# Pull Request Template

## Overview

<!-- Briefly describe the purpose and scope of this PR. What problem does it solve or what feature does it add? -->

---

## Changes

- [ ] New CLI command
- [ ] Enhancement to existing functionality
- [ ] Bug fix
- [ ] Documentation update
- [ ] Refactoring
- [ ] Dependency update
- [ ] Other (describe below)

---

## Checklist

- [ ] All code is written in English
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated (`docs/`, `docs/dev/`)
- [ ] Progress and roadmap updated (`docs/dev/progress.md`, `docs/dev/development-roadmap.md`)
- [ ] Commit messages are clear and in English
- [ ] Pull request includes a summary of changes and test results

---

## Development & Testing

- [ ] Feature requirements are documented (if applicable)
- [ ] Tests are written before implementation (TDD)
- [ ] Integration tests updated or added (`tests/integration_tests.rs`)
- [ ] Manual test steps (if needed):
  ```bash
  cargo build
  cargo run -- [command] [args]
  cargo test
  ```

---

## Related Issues
- Closes #
- Related to #

---

## Breaking Changes
- [ ] This PR introduces breaking changes (describe below)
- [x] No breaking changes

---

## Reviewer Notes

<!-- Anything reviewers should pay special attention to, or points for discussion. -->

---

Thank you for contributing to rmz!
