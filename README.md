# rmz

A safe, modern alternative to `rm` for the command line. Instead of permanently deleting files, `rmz` moves them to a TrashZone, allowing for easy recovery and peace of mind.

---

## Features

- **Safe file removal:** Files are moved to a hidden TrashZone (`~/.rmz/trash/`), not destroyed
- **Easy restore:** Recover deleted files with a single command
- **List & browse:** View deleted files and operations (`rmz list`, `--tree`)
- **Permanent delete:** Erase files from TrashZone when you decide (`rmz purge`)
- **Protection:** Prevent deletion of critical files via protected paths
- **Dry-run:** Preview restore or purge actions without making changes
- **UUID-based tracking:** Reliable, collision-free operation IDs

---

## Quick Start

```bash
# Remove a file safely
$ rmz delete main.rs

# List deleted operations
$ rmz list

# View deleted files in tree format
$ rmz list --tree <operation_id>

# Preview restore (dry-run)
$ rmz restore <operation_id> --dry-run

# Restore files
$ rmz restore <operation_id>

# Permanently delete from TrashZone
$ rmz purge <operation_id>
```

---

## Project Structure

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

## Philosophy

- **Safety first:** Never lose files by accident
- **Reversible:** Every deletion can be undone until purged
- **Simple CLI:** Familiar, `rm`-compatible commands
- **Test-driven:** All features are covered by tests

---

## Contribution

- All development is done on feature branches using `git worktree`
- Test-driven development (TDD) is required: write tests before implementation
- Update documentation in `docs/` and `docs/dev/` as needed
- Use English for all code, comments, and documentation
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting code
- See [CLAUDE.md](CLAUDE.md) for detailed contribution guidelines

---

## FAQ

- **How do I recover a deleted file?**
  - Use `rmz list` to find the operation ID, then `rmz restore <operation_id>`
- **How do I permanently delete files?**
  - Use `rmz purge <operation_id>`
- **Where are deleted files stored?**
  - In `~/.rmz/trash/` with metadata for recovery
- **How do I protect important files?**
  - Configure protected paths in the settings file (see docs)

---

## License

MIT
