# rmz Alias Guide

A practical guide for safely replacing the `rm` command with `rmz` using shell aliases and functions.

---

## 1. Setup: Recommended Aliases

### Bash/Zsh (`.bashrc`, `.zshrc`)
```bash
alias rm='rmz delete'
alias rm-dry='rmz delete --dry-run'
alias rm-force='rmz delete --force'
alias rm-interactive='rmz delete --interactive'
alias unrm='rmz restore'
alias unrm-all='rmz restore --all'
alias trash='rmz list'
alias trash-status='rmz status'
alias trash-empty='rmz purge --all'  # if implemented
alias rm-verbose='rmz delete --verbose'
```

### Fish Shell (`~/.config/fish/config.fish`)
```fish
alias rm 'rmz delete'
alias rm-dry 'rmz delete --dry-run'
alias rm-force 'rmz delete --force'
alias unrm 'rmz restore'
alias trash 'rmz list'
alias trash-status 'rmz status'
```

### Tag-based and Safety-focused Aliases
```bash
alias rm-temp='rmz delete --tag temp'
alias rm-backup='rmz delete --tag backup'
alias rm-old='rmz delete --tag old'
alias unrm-temp='rmz restore --tag temp'
alias rm='rmz delete --interactive'  # safer default
```

### Useful Shell Functions (Bash/Zsh)
```bash
rm_and_status() {
    rmz delete "$@" && rmz status
}
alias rms='rm_and_status'

rm_with_quick_restore() {
    rmz delete "$@"
    rmz list --limit 1 | grep -o '[a-f0-9]\{8\}' > ~/.rmz_last_deleted
}
unrm_last() {
    if [ -f ~/.rmz_last_deleted ]; then
        local last_id=$(cat ~/.rmz_last_deleted)
        rmz restore --id "$last_id"
        rm ~/.rmz_last_deleted
    else
        echo "No recent deletion found."
    fi
}
alias rm='rm_with_quick_restore'
alias unrm-last='unrm_last'
```

---

## 2. Usage Examples & Workflows

```bash
rm file.txt                # Move file to TrashZone
rm directory/              # Remove directory safely
rm-force important.conf    # Force move to TrashZone
unrm important.pdf         # Restore by filename
trash                      # List deleted files
unrm --id a1b2c3d4         # Restore by operation ID
```

### Tagging & Automation
```bash
rm-temp *.tmp              # Tag temporary files
rm-backup *.bak            # Tag backup files
find . -name "*.tmp" | xargs rm-temp
```

---

## 3. Troubleshooting

- **`rmz` not found:**
  - Check your PATH: `which rmz`
  - Re-source your shell config: `source ~/.bashrc` or `source ~/.zshrc`
- **Alias not working:**
  - Confirm with `type rm`
  - Use `unalias rm` to remove conflicting aliases
- **Use original `rm`:**
  - Call with backslash: `\rm file.txt` or `/bin/rm file.txt`
- **Restore last deleted:**
  - Use the `unrm-last` function above

---

## 4. Best Practices

- Start with `rm-dry` to preview deletions
- Use tags to document deletion reasons
- Regularly check TrashZone status (`trash-status`)
- Always use `--interactive` for important files
- For scripts, use `rmz delete` explicitly
- In shared environments, agree on alias policies with your team

---

## Security Notes

- For system administration, use the original `rm` when required
- For sensitive files, consider permanent deletion (`rmz purge`)
- Review and update aliases as `rmz` evolves

---

This guide helps you safely and flexibly use `rmz` as your daily file removal tool. 