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

## rmをrmzに置き換える (推奨)

危険な`rm`コマンドを安全な`rmz delete`で置き換えることができます：

### 自動セットアップ

```bash
./scripts/setup-rm-alias.sh
```

### 手動セットアップ

```bash
# シェル設定ファイルにエイリアスを追加
echo "alias rm='rmz delete'" >> ~/.bashrc  # bashの場合
echo "alias rm='rmz delete'" >> ~/.zshrc   # zshの場合

# 設定を再読み込み
source ~/.bashrc  # または ~/.zshrc
```

### 使用方法

エイリアス設定後は、通常の`rm`コマンドがrmzに置き換わります：

```bash
rm file.txt           # ファイルをトラッシュに移動
rm -r directory/      # ディレクトリを再帰的にトラッシュに移動
rm -f important.txt   # 強制削除でもトラッシュに移動（安全）

# ファイルの復元
rmz restore --interactive

# 完全削除
rmz purge --days 30
```

詳細は [docs/rm-alias-setup.md](docs/rm-alias-setup.md) を参照してください。

## ライセンス

MIT OR Apache-2.0