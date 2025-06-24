# rmz Alias Usage Examples

rmzをrmコマンドの代替として使用する具体的な例を示します。

## 基本的な使用例

### 従来のrmコマンド vs rmz alias

```bash
# 従来のrmコマンド
rm file.txt                    # ❌ 永久削除
rm -r directory/               # ❌ ディレクトリ永久削除
rm -f important.conf           # ❌ 強制永久削除

# rmz aliasでの安全な削除
rm file.txt                    # ✅ trashに移動
rm directory/                  # ✅ ディレクトリも安全に削除
rm-force important.conf        # ✅ 確認なしでtrashに移動
```

### 復元機能の活用

```bash
# ファイルを誤って削除してしまった場合
rm important_document.pdf

# すぐに復元可能
unrm important_document.pdf    # ファイル名で復元
# または
trash                          # 削除したファイル一覧を確認
unrm --id a1b2c3d4             # IDで復元
```

## 実際のワークフロー例

### 開発環境での使用

```bash
# プロジェクトのクリーンアップ
rm-temp *.tmp                  # 一時ファイルをtempタグ付きで削除
rm-backup *.bak                # バックアップファイルを削除
rm target/                     # ビルド結果を削除

# 削除内容の確認
trash                          # 削除したファイルを確認
trash-status                   # trash容量確認

# 必要に応じて復元
unrm-last                      # 最後に削除したファイルを復元
```

### システム管理での使用

```bash
# 設定ファイルの更新
cp nginx.conf nginx.conf.backup
rm-backup nginx.conf.old       # 古い設定をバックアップタグで削除

# ログファイルの管理
rm-old /var/log/*.log.old      # 古いログを削除

# 削除前の確認（ドライラン）
rm-dry /tmp/*                  # 実際に削除せず、対象ファイルを表示
rm /tmp/*                      # 確認後に実際に削除
```

### マルチユーザー環境

```bash
# ユーザー別の削除
rm-tag="user:alice" ~/documents/old_files/
rm-tag="cleanup:weekly" /shared/temp/

# 削除理由の追跡
trash                          # タグ付きで削除履歴を確認
```

## 高度な使用パターン

### パイプラインでの使用

```bash
# 古いファイルを検索して削除
find . -name "*.tmp" -mtime +7 | xargs rm-temp

# 大容量ファイルの削除
du -h . | sort -hr | head -10   # 大容量ファイルを特定
rm-backup large_file.iso        # バックアップタグで削除
```

### スクリプトでの活用

```bash
#!/bin/bash
# プロジェクトクリーンアップスクリプト

echo "🧹 プロジェクトクリーンアップ開始"

# ビルド生成物の削除
rm-tag="build" target/ dist/ *.o

# 一時ファイルの削除
rm-temp *.tmp *.swp *~

# 削除結果の表示
echo "📊 削除完了:"
trash-status

echo "復元が必要な場合は 'trash' で確認し、'unrm --id <ID>' で復元できます"
```

### Git操作との連携

```bash
# Git管理外ファイルの削除
git clean -n                   # Git clean予行演習
git ls-files --others --ignored --exclude-standard | xargs rm-gitignore

# ブランチ切り替え前のクリーンアップ
rm-temp *.pyc __pycache__/
git checkout main
```

## エラーハンドリング例

### 権限エラーの対処

```bash
# 権限のないファイルを削除しようとした場合
rm /etc/important.conf         # 権限エラー

# rmzの場合
rm /etc/important.conf         # エラー表示されるが、システムは安全
sudo rm-force /etc/important.conf  # 必要に応じてsudoで実行
```

### 復旧手順

```bash
# 大量削除してしまった場合
rm -rf project/                # 大量削除

# 確認と復旧
trash                          # 削除内容確認
unrm project/                  # ディレクトリ全体を復元

# または段階的復旧
unrm --interactive             # 対話的に選択して復元
```

## トラブルシューティング

### よくある問題と解決法

```bash
# 1. rmzコマンドが見つからない
type rm                        # alias確認
which rmz                      # rmzの場所確認
export PATH="$PATH:/path/to/rmz"

# 2. aliasが効かない
source ~/.bashrc               # 設定再読み込み
alias rm                       # alias定義確認

# 3. 元のrmコマンドを使いたい
rm-original file.txt           # 元のrmコマンド使用
\rm file.txt                   # alias回避
/bin/rm file.txt               # 直接実行

# 4. aliasを無効化したい
rm-disable                     # rmz aliasを無効化
unalias rm                     # 手動でalias削除
```

### パフォーマンス比較

```bash
# 大量ファイル削除のパフォーマンステスト
time rm-original huge_directory/     # 従来rm
time rm huge_directory/              # rmz

# 通常はrmzの方がrename操作なので高速
```

## ベストプラクティス

### 推奨設定

1. **段階的導入**: まず`rm-dry`で習慣化
2. **タグ活用**: 削除理由を記録
3. **定期確認**: `trash-status`で容量監視
4. **バックアップ**: 重要削除前は`rm-backup`タグ
5. **スクリプト**: 自動化でも`--tag`を活用

### セキュリティ考慮事項

- システム管理では元の`rm`コマンドも併用
- 機密ファイルは適切な永久削除も検討
- 共有環境では削除ポリシーを明確化

この設定により、rmzを日常的にrmコマンドの安全な代替として活用できます。