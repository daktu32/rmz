# rmz Alias Guide

rmzをrmコマンドの安全な代替として導入・活用するための総合ガイドです。

---

## 1. セットアップ手順（alias設定・推奨設定）

### Bash/Zsh (.bashrc, .zshrc)
```bash
alias rm='rmz delete'
alias rm-dry='rmz delete --dry-run'
alias rm-force='rmz delete --force'
alias rm-interactive='rmz delete --interactive'
alias rm-r='rmz delete'
alias rm-rf='rmz delete --force'
alias unrm='rmz restore'
alias unrm-all='rmz restore --all'
alias unrm-interactive='rmz restore --interactive'
alias trash='rmz list'
alias trash-status='rmz status'
alias trash-empty='rmz purge --all'  # 将来実装予定
alias rm-verbose='rmz delete --verbose'
alias rm-tag='rmz delete --tag'
```

### Fish Shell (~/.config/fish/config.fish)
```fish
alias rm 'rmz delete'
alias rm-dry 'rmz delete --dry-run'
alias rm-force 'rmz delete --force'
alias rm-interactive 'rmz delete --interactive'
alias unrm 'rmz restore'
alias unrm-all 'rmz restore --all'
alias trash 'rmz list'
alias trash-status 'rmz status'
```

### タグ付き削除・安全性重視の設定
```bash
alias rm-temp='rmz delete --tag temporary'
alias rm-backup='rmz delete --tag backup'
alias rm-old='rmz delete --tag old-files'
alias unrm-temp='rmz restore --tag temporary'
alias unrm-backup='rmz restore --tag backup'
alias rm='rmz delete --interactive'
alias rm-important='rmz delete --interactive --tag important'
alias rm-safe='rmz delete --dry-run'
alias rm-execute='rmz delete'
```

### 便利な関数定義（Bash/Zsh例）
```bash
rm_and_status() {
    rmz delete "$@" && rmz status
}
alias rms='rm_and_status'
rm_with_quick_restore() {
    local files=("$@")
    rmz delete "${files[@]}"
    echo "削除完了。復元するには: unrm-last"
    rmz list --limit 1 | grep -o '[a-f0-9]\{8\}' > ~/.rmz_last_deleted
}
unrm_last() {
    if [ -f ~/.rmz_last_deleted ]; then
        local last_id=$(cat ~/.rmz_last_deleted)
        rmz restore --id "$last_id"
        rm ~/.rmz_last_deleted
    else
        echo "最近削除したファイルがありません"
    fi
}
alias rm='rm_with_quick_restore'
alias unrm-last='unrm_last'
```

---

## 2. 活用例・ワークフロー

### 基本的な使用例
```bash
rm file.txt                    # trashに移動
rm directory/                  # ディレクトリも安全に削除
rm-force important.conf        # 確認なしでtrashに移動
unrm important_document.pdf    # ファイル名で復元
trash                          # 削除したファイル一覧を確認
unrm --id a1b2c3d4             # IDで復元
```

### 開発・運用ワークフロー例
```bash
rm-temp *.tmp                  # 一時ファイルをtempタグ付きで削除
rm-backup *.bak                # バックアップファイルを削除
rm target/                     # ビルド結果を削除
trash                          # 削除したファイルを確認
trash-status                   # trash容量確認
unrm-last                      # 最後に削除したファイルを復元
```

### システム管理・マルチユーザー環境
```bash
rm-backup nginx.conf.old       # 古い設定をバックアップタグで削除
rm-old /var/log/*.log.old      # 古いログを削除
rm-dry /tmp/*                  # ドライラン
rm-tag="user:alice" ~/documents/old_files/
rm-tag="cleanup:weekly" /shared/temp/
```

### パイプライン・スクリプト・Git連携
```bash
find . -name "*.tmp" -mtime +7 | xargs rm-temp
rm-tag="build" target/ dist/ *.o
rm-temp *.pyc __pycache__/
git ls-files --others --ignored --exclude-standard | xargs rm-gitignore
```

---

## 3. トラブルシューティング・注意事項

### よくある問題と解決法
```bash
# rmzコマンドが見つからない
type rm
which rmz
export PATH="$PATH:/path/to/rmz"
# aliasが効かない
source ~/.bashrc
alias rm
type rm
# 元のrmコマンドを使いたい
rm-original file.txt
\rm file.txt
/bin/rm file.txt
# aliasを無効化したい
rm-disable
unalias rm
```

### エラーハンドリング・復旧
```bash
sudo rm-force /etc/important.conf  # 権限が必要な場合
trash                              # 削除内容確認
unrm project/                      # ディレクトリ全体を復元
unrm --interactive                 # 対話的に選択して復元
```

### 注意事項
- スクリプト内では`rmz delete`を明示的に使用
- システム管理作業では元の`rm`コマンドも併用
- チーム環境では事前に合意を取る
- 重要な削除前は`--dry-run`で確認

---

## 4. ベストプラクティス・推奨運用

1. **段階的導入**: まず`rm-dry`で習慣化
2. **タグ活用**: 削除理由を記録
3. **定期確認**: `trash-status`で容量監視
4. **バックアップ**: 重要削除前は`rm-backup`タグ
5. **スクリプト**: 自動化でも`--tag`を活用

### セキュリティ考慮事項
- システム管理では元の`rm`コマンドも併用
- 機密ファイルは適切な永久削除も検討
- 共有環境では削除ポリシーを明確化

---

このガイドにより、rmzを日常的に安全かつ柔軟に活用できます。 