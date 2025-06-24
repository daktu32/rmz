# rmz

The next gen rm command

## 概要

rmz は Rust で開発された CLI ツールです。

## インストール

### Cargo を使用してインストール

```bash
cargo install rmz
```

### ソースからビルド

```bash
git clone https://github.com/daktu32/rmz
cd rmz
cargo build --release
```

## 使用方法

### 基本コマンド

```bash
# ヘルプを表示
rmz --help

# 例コマンドを実行
rmz example

# 詳細モードで実行
rmz example --verbose
# 既存ファイルを強制上書きで復元
rmz restore --id <uuid> --force
# 既存ファイルがある場合は自動リネーム
rmz restore --id <uuid> --rename
```

## 開発

### 前提条件

- Rust 1.70 以上

### セットアップ

```bash
git clone https://github.com/daktu32/rmz
cd rmz
cargo build
```

### テスト実行

```bash
cargo test
```

### フォーマットとリント

```bash
cargo fmt
cargo clippy
```

## ライセンス

MIT OR Apache-2.0