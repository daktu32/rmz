# Product Requirements Document (PRD)

This document defines the product requirements for the rmz Rust CLI project.

---

## Purpose
- Provide a safe, modern replacement for the `rm` command
- Prevent accidental data loss and enable easy file recovery
- Offer a user-friendly, cross-platform CLI experience

---

## Core Requirements

### 1. Safe Deletion
- Files are never permanently deleted by default
- Deleted files are moved to a dedicated TrashZone (`~/.rmz/trash/`)
- Metadata is recorded for each deletion (JSON)

### 2. Easy Restore
- Users can restore files by ID, name, or interactively
- Restored files return to their original location

### 3. Listing & Status
- List all deleted files and operations
- Show trash statistics and health status

### 4. Permanent Deletion
- Users can permanently delete files from TrashZone (`rmz purge`)
- Disk space management and retention policies

### 5. Protection & Configuration
- Prevent deletion of protected paths
- Configurable settings (protected paths, auto-clean, max size)

### 6. Usability
- Familiar CLI interface (clap-based)
- Helpful error messages and confirmations
- English output and documentation

### 7. Quality & Security
- Test-driven development (TDD)
- Comprehensive test coverage
- No clippy warnings or security vulnerabilities
- CI/CD for all changes

---

## Non-Goals
- No web UI or server components
- No cloud or database dependencies
- No non-Rust runtime requirements

---

## References
- See `docs/dev/architecture.md` for system design
- See `docs/dev/test-strategy.md` for testing approach
