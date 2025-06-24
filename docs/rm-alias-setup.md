# rmz as rm Replacement Setup

This guide shows how to replace the dangerous `rm` command with the safe `rmz delete` command.

## Quick Setup (Recommended)

Run the automated setup script:

```bash
./scripts/setup-rm-alias.sh
```

This will:
- Build rmz if needed
- Install rmz to system PATH
- Add alias to your shell configuration
- Provide usage instructions

## Manual Setup

### 1. Install rmz

```bash
# Build rmz
cargo build --release

# Install to system (requires sudo)
sudo ln -sf $(pwd)/target/release/rmz /usr/local/bin/rmz

# Or add to your PATH in shell config
export PATH="$PATH:$(pwd)/target/release"
```

### 2. Add Alias to Shell

#### Bash (.bashrc or .bash_profile)
```bash
echo "alias rm='rmz delete'" >> ~/.bashrc
source ~/.bashrc
```

#### Zsh (.zshrc)
```bash
echo "alias rm='rmz delete'" >> ~/.zshrc
source ~/.zshrc
```

#### Fish (.config/fish/config.fish)
```bash
echo "alias rm='rmz delete'" >> ~/.config/fish/config.fish
```

## Usage After Setup

Once the alias is active:

```bash
# These now use rmz delete instead of rm
rm file.txt                    # Moves file to trash
rm -r directory/               # Moves directory to trash recursively
rm -f important.txt            # Moves file to trash (force flag works)
rm -rf project/                # Moves directory to trash recursively

# Restore deleted files
rmz restore --interactive      # Interactive restore
rmz restore --id a1b2c3d4      # Restore specific file

# View trash contents
rmz list                       # List all trashed files
rmz status                     # Show trash statistics

# Permanently delete
rmz purge --days 30            # Delete files older than 30 days
rmz purge --all                # Delete everything permanently
```

## Safety Features

- **No accidental deletion**: Files are moved to trash, not permanently deleted
- **Protected paths**: System directories are protected by default
- **Confirmation prompts**: Interactive mode for sensitive operations
- **Operation logging**: All operations are logged for audit
- **Partial UUID restore**: Easy file identification with short IDs

## Bypass Alias (Original rm)

If you need the original `rm` behavior:

```bash
command rm file.txt            # Use original rm
/bin/rm file.txt               # Direct path to rm
\rm file.txt                   # Escape alias
```

## Configuration

Customize rmz behavior:

```bash
rmz config show               # Show current configuration
rmz config set colors true    # Enable colored output
rmz config set auto_clean_days 30  # Auto-clean after 30 days
```

## Protect Additional Paths

```bash
rmz protect add ~/important   # Protect directory from deletion
rmz protect list              # Show protected paths
```

## Troubleshooting

### Alias not working
```bash
# Check if alias exists
alias rm

# Reload shell configuration
source ~/.bashrc  # or ~/.zshrc

# Check rmz is in PATH
which rmz
```

### Permission issues
```bash
# If you can't write to /usr/local/bin
mkdir -p ~/bin
ln -sf $(pwd)/target/release/rmz ~/bin/rmz
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
```

### Revert to original rm
```bash
# Remove alias from shell config
sed -i '/alias rm=/d' ~/.bashrc
source ~/.bashrc
```

## Why Replace rm?

- **Prevent data loss**: No more accidental `rm -rf /`
- **Recoverable deletions**: Restore files easily
- **Better UX**: Colored output, progress indicators
- **Audit trail**: Track all deletion operations
- **Smart defaults**: Sensible behavior out of the box

The alias approach means you keep your muscle memory while gaining safety!