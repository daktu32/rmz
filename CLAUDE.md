# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wezterm-parallel - 

## Technology Stack

See (docs/tech-stack.md) for complete technology stack definitions and rationale.

## Architecture

### Overview
```

Example:
 →  →  →  → 
```

### Key Components
- **Frontend**: 
- **API Layer**: 
- **Business Logic**: 
- **Data Layer**: 

## Development Philosophy

### Core Principles
- **Test-Driven Development (TDD)**: Write tests first, then implement
- **Clean Code**: Maintainable, readable, and well-documented
- **Security First**: Follow security best practices from the start
- **Performance**: Optimize for speed and efficiency
- **Scalability**: Design for growth from day one

## Key Features

### MVP Features
1. ****: 
2. ****: 
3. ****: 
4. ****: 
5. ****: 

### Future Features
- 
- 
- 

## Security & Compliance

### Security Measures
- **Authentication**: 
- **Authorization**: 
- **Data Protection**: 
- **Network Security**: 

### Compliance Considerations
- **Data Privacy**: 
- **Legal Requirements**: 
- **Industry Standards**: 

## Cost Structure

### Estimated Costs
- **Development Phase**: $/month
- **MVP Phase**: $/month
- **Production Phase**: $/month

### Cost Breakdown
- **Infrastructure**: 
- **Third-party Services**: 
- **Scaling Factors**: 

## Development Workflow

### Current Status
- **Phase**: 
- **Sprint**: 
- **Milestone**: 

### Active Development
1. 
2. 
3. 

## Progress Management Rules

### Required File Updates
AI agents must keep the following files up to date:

1. **PROGRESS.md** - Development progress tracking
   - Update after completing each task
   - Document completed tasks, current work, and next tasks
   - Include dates and timestamps

2. **DEVELOPMENT_ROADMAP.md** - Development roadmap
   - Update as phases progress
   - Mark completed milestones with checkmarks
   - Reflect new challenges or changes

### Update Timing
- Upon feature implementation completion
- After important configuration changes
- During phase transitions
- After bug fixes or improvements
- When making new technical decisions

### Update Method
1. Update relevant files immediately after work completion
2. Document specific deliverables and changes
3. Clarify next steps
4. Include progress updates in commit messages

## Project-Specific Development Rules

### Git Workflow

#### Branch Strategy
- **Main Branch**: `main`
- **Feature Branches**: `feature/task-description`
- **Bug Fix Branches**: `fix/bug-description`

#### Required Work Procedures
Follow these steps for all development work:

1. Define feature requirements and document in `docs/specs/`
2. **Create work branch and isolate with git worktree**
3. Create tests based on expected inputs and outputs
4. Run tests and confirm failures
5. Implement code to pass tests
6. Refactor once all tests pass
7. Update progress files (PROGRESS.md, DEVELOPMENT_ROADMAP.md)

#### Worktree Usage
```bash
# Required steps
git checkout main && git pull origin main
git checkout -b feature/task-name
git worktree add ../project-feature ./feature/task-name
```

### Module Structure

- `packages/frontend/`: Frontend application
- `packages/backend/`: Backend services
- `packages/shared/`: Shared utilities and types
- `infrastructure/`: Infrastructure as Code
- `docs/`: Documentation
- `scripts/`: Utility scripts

### Coding Standards

#### File Naming Conventions
- **Components**: `PascalCase.tsx` (React/Vue)
- **Utilities**: `camelCase.ts`
- **API Handlers**: `kebab-case.ts`
- **Test Files**: `*.test.ts(x)` or `*.spec.ts(x)`
- **Type Definitions**: `*.types.ts`

#### Quality Checklist
実装完了前に以下を確認：
- `npm run type-check` (TypeScript validation)
- `npm run lint` (ESLint + Prettier)
- `npm run test` (Jest tests pass)
- `npm run build` (Production build succeeds)

### Cloud Integration Guidelines

#### Service Architecture
- **Authentication**: 
- **Database**: 
- **Storage**: 
- **Compute**: 
- **CDN**: 

#### Security Principles
- Principle of least privilege
- Secrets management
- Secure communication (HTTPS/TLS)
- Regular security audits

### Prohibited Practices

The following practices are strictly prohibited:
- Implementing features without tests
- Working directly on the main branch
- Hardcoding secrets or credentials
- Breaking existing API interfaces
- Adding external dependencies without approval
- Skipping documentation updates
- Ignoring PROGRESS.md and DEVELOPMENT_ROADMAP.md updates

### Post-Implementation Checklist
-  All tests pass
-  Type checking passes
-  Linting passes
-  Documentation updated
-  PROGRESS.md updated with completed and next tasks
-  DEVELOPMENT_ROADMAP.md updated with progress
-  Changes committed with descriptive message
-  Pull Request created with clear description
