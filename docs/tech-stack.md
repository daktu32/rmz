# Technology Stack

This document defines the technology stack for this project. Other documentation files reference this as the single source of truth.

## Core Technologies

### Language
- **Primary**: Rust
- **Version**: 1.75+
- **Rationale**: メモリ安全性、高速性、シングルバイナリ配布

### CLI Framework
- **Primary**: clap v4
- **Features**: derive macro, 自動補完生成
- **Rationale**: Rust製CLIの定番、型安全で宣言的

### Output Formatting
- **Coloring**: colored または owo-colors
- **Interactive**: dialoguer
- **Table Display**: comfy-table (将来的に)

## File Management

### Trash Management
- **Library**: trash-rs (将来的に) / 独自実装
- **File Operations**: fs_extra
- **Path Handling**: path-absolutize

### Unique Identifiers
- **UUID Generation**: uuid v4
- **Purpose**: ファイルIDの一意化、衝突回避

### Date/Time
- **Library**: chrono
- **Format**: RFC3339 (JSON保存用)

## Data Storage

### Metadata Storage
- **Primary**: JSON (serde + serde_json)
- **Alternative**: SQLite (rusqlite) - 将来的な拡張用
- **Location**: XDG準拠 (~/.local/share/rmz/)

### Configuration
- **Format**: TOML
- **Library**: confy / toml
- **File**: ~/.config/rmz/config.toml

### Directory Management
- **Library**: directories
- **Purpose**: クロスプラットフォーム対応のパス取得

## Interactive Features

### Fuzzy Finder Integration
- **External**: fzf (duct経由で呼び出し)
- **Built-in Alternative**: skim
- **Use Cases**: restore時のファイル選択

### External Command Execution
- **Library**: duct
- **Purpose**: fzf, less等の外部ツール連携

### Terminal UI (将来的に)
- **Library**: ratatui
- **Purpose**: TUIモードの実装

## DevOps & CI/CD

### Version Control
- **Platform**: GitHub
- **Workflow**: GitHub Flow (main + feature branches)

### CI/CD Pipeline
- **Platform**: GitHub Actions
- **Jobs**: test, clippy, build, release

### Release Management
- **Tool**: cargo-release
- **Binary Distribution**: GitHub Releases
- **Changelog**: git-cliff

## Development Tools

### Code Quality
- **Linting**: clippy
- **Formatting**: rustfmt
- **Type Checking**: cargo check

### Testing
- **Unit Testing**: 内蔵 #[test]
- **Integration Testing**: tests/ ディレクトリ
- **CLI Testing**: assert_cmd
- **Snapshot Testing**: insta
- **Test Utilities**: tempfile, mockall

### Documentation
- **API Docs**: cargo doc
- **User Docs**: mdBook
- **Examples**: examples/ ディレクトリ

## Security

### File Permissions
- **Preservation**: メタデータで元の権限を保存
- **TrashZone**: ユーザー専用領域 (700)

### Path Protection
- **Protected Paths**: /etc, /usr, ~/.ssh等
- **Configuration**: ~/.config/rmz/protected_paths.toml

### Safe Defaults
- **Confirmation**: 重要操作時の確認プロンプト
- **Dry Run**: --dry-runオプション必須

## Distribution & Packaging

### Package Managers
- **Cargo**: crates.io公開
- **Homebrew**: Formula作成
- **AUR**: PKGBUILD提供
- **Debian/Ubuntu**: .deb (cargo-deb)

### Binary Distribution
- **Cross Compilation**: cross
- **Targets**: x86_64, aarch64
- **Compression**: tar.gz, zip

### Shell Completion
- **Generator**: clap_complete
- **Shells**: bash, zsh, fish, powershell

## Version Requirements

| Technology | Minimum Version | Recommended Version | Notes |
|------------|----------------|-------------------|-------|
| Rust | 1.70 | 1.75+ | async/await, const generics |
| clap | 4.0 | 4.4+ | derive macro improvements |
| tokio | 1.0 | 1.35+ | 非同期処理用 (将来的に) |

## Decision Rationale

### Why These Technologies?

1. **Rust**: メモリ安全性と高速性の両立、シングルバイナリ配布
2. **clap**: 型安全なCLI構築、自動ドキュメント生成
3. **JSON + serde**: シンプルで可読性が高く、デバッグが容易

### Alternative Considerations

| Technology | Alternative Considered | Why Not Chosen |
|------------|----------------------|----------------|
| Go | 実装検討 | Rustの方がメモリ安全性が高い |
| SQLite | 初期実装 | JSONの方がシンプルで透明性が高い |
| structopt | CLI framework | clapに統合されたため |

## Migration Path

### MVP → Production

1. **Phase 1**: 基本機能 (delete/restore/list) with JSON
2. **Phase 2**: fzf統合、補完機能、パッケージング
3. **Phase 3**: SQLiteオプション、Web UI検討

## Dependencies

### Critical Dependencies
- clap: CLI構築の中核
- serde/serde_json: データ永続化
- chrono: タイムスタンプ管理
- uuid: ファイル識別子

### Optional Dependencies
- skim/fzf: インタラクティブ選択
- trash-rs: OS標準ゴミ箱統合
- rusqlite: 高度な検索機能

---

**Last Updated**: 2025-06-23  
**Reviewed By**: rmz Development Team  
**Next Review**: v0.2.0 リリース時