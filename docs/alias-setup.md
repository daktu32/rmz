# rmz Alias Setup Guide

rmzをrmコマンドの完全な代替として使用するためのalias設定ガイドです。

## 基本的なrm代替

### Bash/Zsh (.bashrc, .zshrc)

```bash
# rmz を rm の安全な代替として使用
alias rm='rmz delete'
alias rm-dry='rmz delete --dry-run'
alias rm-force='rmz delete --force'
alias rm-interactive='rmz delete --interactive'

# 再帰削除（現在はディレクトリもそのまま削除可能）
alias rm-r='rmz delete'
alias rm-rf='rmz delete --force'

# 復元・管理機能
alias unrm='rmz restore'
alias unrm-all='rmz restore --all'
alias unrm-interactive='rmz restore --interactive'
alias trash='rmz list'
alias trash-status='rmz status'
alias trash-empty='rmz purge --all'  # 将来実装予定

# 詳細情報付きコマンド
alias rm-verbose='rmz delete --verbose'
alias rm-tag='rmz delete --tag'
```

### Fish Shell (~/.config/fish/config.fish)

```fish
# rmz aliases for fish shell
alias rm 'rmz delete'
alias rm-dry 'rmz delete --dry-run'
alias rm-force 'rmz delete --force'
alias rm-interactive 'rmz delete --interactive'

# 復元・管理
alias unrm 'rmz restore'
alias unrm-all 'rmz restore --all'
alias trash 'rmz list'
alias trash-status 'rmz status'
```

## 高度な使用例

### タグ付き削除
```bash
# プロジェクト別の削除
alias rm-temp='rmz delete --tag temporary'
alias rm-backup='rmz delete --tag backup'
alias rm-old='rmz delete --tag old-files'

# 復元時のタグフィルタ（将来実装予定）
alias unrm-temp='rmz restore --tag temporary'
alias unrm-backup='rmz restore --tag backup'
```

### 安全性重視の設定
```bash
# デフォルトで対話モードを使用
alias rm='rmz delete --interactive'

# 重要ファイル用の特別な削除（強制的に確認）
alias rm-important='rmz delete --interactive --tag important'

# ドライランをデフォルトにして、確認後に実行
alias rm-safe='rmz delete --dry-run'
alias rm-execute='rmz delete'
```

## 便利な関数定義

### Bash/Zsh関数

```bash
# ファイル削除後に即座にtrash状況を表示
rm_and_status() {
    rmz delete "$@" && rmz status
}
alias rms='rm_and_status'

# 削除したファイルをすぐに確認・復元できる関数
rm_with_quick_restore() {
    local files=("$@")
    rmz delete "${files[@]}"
    echo "削除完了。復元するには: unrm-last"
    # 最後に削除したファイルのIDを記録
    rmz list --limit 1 | grep -o '[a-f0-9]\{8\}' > ~/.rmz_last_deleted
}

# 最後に削除したファイルを復元
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

### Fish関数

```fish
# Fish function for rm with status
function rm_and_status
    rmz delete $argv
    and rmz status
end
alias rms 'rm_and_status'

# Quick restore function
function unrm_last
    set last_id (rmz list --limit 1 | grep -o '[a-f0-9]\{8\}')
    if test -n "$last_id"
        rmz restore --id $last_id
    else
        echo "最近削除したファイルがありません"
    end
end
```

## 推奨設定（段階的導入）

### レベル1: 基本的な代替
```bash
# 最低限のalias設定
alias rm='rmz delete'
alias unrm='rmz restore'
alias trash='rmz list'
```

### レベル2: 安全性向上
```bash
# 対話モードをデフォルトに
alias rm='rmz delete --interactive'
alias rm-force='rmz delete --force'
alias rm-dry='rmz delete --dry-run'
```

### レベル3: 完全な統合
```bash
# すべての機能を活用
alias rm='rmz delete --interactive'
alias rm-f='rmz delete --force'
alias rm-rf='rmz delete --force'
alias rm-i='rmz delete --interactive'
alias rm-v='rmz delete --verbose'

alias unrm='rmz restore --interactive'
alias unrm-all='rmz restore --all'
alias trash='rmz list'
alias trash-status='rmz status'
alias trash-clean='rmz purge --days 30'  # 将来実装
```

## 移行戦略

### 1. 段階的移行
```bash
# 週1: rmzコマンドに慣れる
# rmzコマンドを直接使用

# 週2: 基本aliasを導入
alias rm='rmz delete'
alias unrm='rmz restore'

# 週3: 高度な機能を追加
# タグ機能、対話モードを活用

# 週4: 完全な代替として確立
# すべてのrmオプションをrmzで置き換え
```

### 2. バックアップ戦略
```bash
# 元のrmコマンドを保持
alias rm-original='/bin/rm'
alias rm-real='/bin/rm'

# 緊急時の復帰
alias rm-disable='unalias rm && echo "rmz alias disabled"'
```

## 注意事項

1. **スクリプト内での使用**: スクリプト内では`rmz delete`を明示的に使用
2. **システム管理**: システム管理作業では元の`rm`コマンドを使用することを検討
3. **チーム共有**: チーム環境では事前に合意を取る
4. **バックアップ**: 重要な削除前は`--dry-run`で確認

## トラブルシューティング

### rmzが見つからない場合
```bash
# パスの確認
which rmz
export PATH="$PATH:/path/to/rmz/binary"
```

### alias設定が効かない場合
```bash
# 設定の再読み込み
source ~/.bashrc  # または ~/.zshrc

# alias確認
alias rm
type rm
```

この設定により、rmzを完全にrmコマンドの代替として使用できます。