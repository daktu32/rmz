# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rmz - 安全な削除を実現するモダンなCLIツール

## Technology Stack

See [docs/tech-stack.md](docs/tech-stack.md) for complete technology stack definitions and rationale.

### Key Technologies
- **Language**: Rust
- **CLI Framework**: clap v4
- **Async Runtime**: tokio
- **File Operations**: std::fs + trash-rs
- **Configuration**: TOML (toml crate)
- **Testing**: cargo test + mockall

## Architecture

### Overview
```
rmz - Rust製の安全なファイル削除ツール

削除フロー:
ユーザー → rmz delete → TrashZone → メタデータ記録 → 復元可能

主要コンポーネント:
- CLI層: コマンドパース、ユーザーインタラクション
- ビジネス層: 削除・復元・履歴管理ロジック
- データ層: TrashZone管理、メタデータ永続化
```

### Key Components
- **CLI Interface**: clap によるコマンドパース、対話的UI
- **Core Logic**: ファイル操作、メタデータ管理、履歴追跡
- **Storage**: TrashZone (~/.rmz/trash/)、メタデータJSON
- **Configuration**: ユーザー設定、保護リスト管理

## Development Philosophy

### Core Principles
- **Test-Driven Development (TDD)**: Write tests first, then implement
- **Clean Code**: Maintainable, readable, and well-documented
- **Security First**: Follow security best practices from the start
- **Performance**: Optimize for speed and efficiency
- **Scalability**: Design for growth from day one

## Key Features

### MVP Features
1. **安全な削除 (rmz delete)**: ファイルをTrashZoneに移動し、メタデータを記録
2. **ファイル復元 (rmz restore)**: 削除したファイルを元の場所に復元
3. **履歴表示 (rmz list)**: 削除ファイル一覧をカラフルに表示
4. **永久削除 (rmz purge)**: TrashZone内のファイルを完全削除
5. **保護リスト**: 重要ファイルの削除を防止

### Future Features
- rmz watch: ファイル削除の監視と通知
- Web UI: ブラウザベースの履歴管理インターフェース
- クラウドバックアップ: S3/Dropbox連携

## Security & Compliance

### Security Measures
- **File Permissions**: 元のファイル権限を保持
- **Access Control**: TrashZone はユーザー専用領域
- **Data Protection**: 設定ファイルは600権限で保護
- **Safe Defaults**: 重要システムファイルはデフォルトで保護

### Compliance Considerations
- **Data Privacy**: ユーザーデータはローカルのみ保存
- **Legal Requirements**: OSS ライセンス (MIT/Apache 2.0)
- **Industry Standards**: Rust セキュリティベストプラクティス準拠

## Cost Structure

### Estimated Costs
- **Development Phase**: $0/month (OSS)
- **MVP Phase**: $0/month (GitHub無料枠)
- **Production Phase**: $0/month (静的サイトホスティング)

### Cost Breakdown
- **Infrastructure**: GitHub (無料)
- **Third-party Services**: CI/CD (GitHub Actions無料枠)
- **Scaling Factors**: ダウンロード数に関わらず無料

## Development Workflow

### Current Status
- **Phase**: 初期開発
- **Sprint**: MVP機能実装
- **Milestone**: v0.1.0リリース準備

### Active Development
1. 基本的な削除・復元機能の実装
2. CLIインターフェースの設計
3. テストカバレッジの向上

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

- `src/`: Rustソースコード
  - `src/bin/`: バイナリエントリポイント
  - `src/commands/`: サブコマンド実装
  - `src/core/`: コアビジネスロジック
  - `src/storage/`: TrashZone管理
- `tests/`: 統合テスト
- `docs/`: ドキュメント
- `scripts/`: ビルド・リリーススクリプト

### Coding Standards

#### File Naming Conventions
- **Modules**: `snake_case.rs`
- **Structs/Enums**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Test Files**: `mod tests` in same file

#### Quality Checklist
実装完了前に以下を確認：
- `cargo check` (型チェック)
- `cargo clippy` (Lintチェック)
- `cargo test` (全テストパス)
- `cargo build --release` (リリースビルド成功)

### Cloud Integration Guidelines

#### Service Architecture
- **Binary Distribution**: GitHub Releases
- **Package Managers**: Homebrew, Cargo, AUR
- **Documentation**: GitHub Pages
- **CI/CD**: GitHub Actions
- **Community**: GitHub Issues/Discussions

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
-  cargo test 全パス
-  cargo clippy 警告なし
-  cargo fmt 実行済み
-  ドキュメント更新済み
-  PROGRESS.md 更新済み（完了タスクと次のタスク）
-  DEVELOPMENT_ROADMAP.md 進捗反映済み
-  意味のあるコミットメッセージ
-  明確な説明付きPull Request作成
