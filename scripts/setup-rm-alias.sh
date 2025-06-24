#!/bin/bash

# Setup rmz as rm replacement
# This script creates an alias to replace rm with rmz delete

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Setting up rmz as rm replacement..."

# Function to add alias to shell config
add_alias_to_file() {
    local config_file="$1"
    local alias_line="alias rm='rmz delete'"
    
    if [ -f "$config_file" ]; then
        # Check if alias already exists
        if grep -q "alias rm=" "$config_file"; then
            echo "⚠️  rm alias already exists in $config_file"
            echo "   Current alias: $(grep "alias rm=" "$config_file")"
            read -p "   Replace with rmz delete? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                # Remove existing rm alias and add new one
                sed -i.bak '/alias rm=/d' "$config_file"
                echo "$alias_line" >> "$config_file"
                echo "✅ Updated rm alias in $config_file"
            else
                echo "⏭️  Skipped $config_file"
            fi
        else
            echo "$alias_line" >> "$config_file"
            echo "✅ Added rm alias to $config_file"
        fi
    else
        echo "📝 Creating $config_file with rm alias"
        echo "$alias_line" > "$config_file"
    fi
}

# Check if rmz is installed or build it
if ! command -v rmz &> /dev/null; then
    echo "🔨 rmz not found in PATH. Building from source..."
    cd "$PROJECT_DIR"
    cargo build --release
    
    # Add to PATH or create symlink
    if [ -w "/usr/local/bin" ]; then
        echo "🔗 Creating symlink in /usr/local/bin"
        ln -sf "$PROJECT_DIR/target/release/rmz" /usr/local/bin/rmz
    else
        echo "⚠️  Cannot write to /usr/local/bin"
        echo "   Please add $PROJECT_DIR/target/release to your PATH"
        echo "   Or run: sudo ln -sf $PROJECT_DIR/target/release/rmz /usr/local/bin/rmz"
        exit 1
    fi
fi

# Detect shell and add alias
current_shell=$(basename "$SHELL")

case "$current_shell" in
    bash)
        # Try .bashrc first, then .bash_profile
        if [ -f "$HOME/.bashrc" ]; then
            add_alias_to_file "$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            add_alias_to_file "$HOME/.bash_profile"
        else
            add_alias_to_file "$HOME/.bashrc"
        fi
        ;;
    zsh)
        add_alias_to_file "$HOME/.zshrc"
        ;;
    fish)
        # Fish uses a different syntax
        fish_config_dir="$HOME/.config/fish"
        mkdir -p "$fish_config_dir"
        fish_alias_file="$fish_config_dir/config.fish"
        
        if [ -f "$fish_alias_file" ]; then
            if grep -q "alias rm=" "$fish_alias_file" || grep -q "function rm" "$fish_alias_file"; then
                echo "⚠️  rm alias/function already exists in $fish_alias_file"
                read -p "   Replace with rmz delete? (y/N): " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    sed -i.bak '/alias rm=/d; /function rm/,/^end$/d' "$fish_alias_file"
                    echo "alias rm='rmz delete'" >> "$fish_alias_file"
                    echo "✅ Updated rm alias in $fish_alias_file"
                else
                    echo "⏭️  Skipped $fish_alias_file"
                fi
            else
                echo "alias rm='rmz delete'" >> "$fish_alias_file"
                echo "✅ Added rm alias to $fish_alias_file"
            fi
        else
            echo "alias rm='rmz delete'" > "$fish_alias_file"
            echo "✅ Created $fish_alias_file with rm alias"
        fi
        ;;
    *)
        echo "⚠️  Unsupported shell: $current_shell"
        echo "   Please manually add this alias to your shell config:"
        echo "   alias rm='rmz delete'"
        ;;
esac

echo
echo "🎉 Setup complete!"
echo
echo "📋 Next steps:"
echo "   1. Restart your shell or run: source ~/.${current_shell}rc"
echo "   2. Test with: rm --help (should show rmz delete help)"
echo "   3. Use rm normally - files will be moved to trash instead of deleted"
echo
echo "🔄 To restore files: rmz restore --interactive"
echo "🗑️  To permanently delete: rmz purge"
echo "📊 To see trash status: rmz status"
echo
echo "⚠️  Important notes:"
echo "   - This replaces rm with rmz delete for safety"
echo "   - Use 'command rm' or '/bin/rm' for original rm behavior"
echo "   - Files are moved to trash zone: ~/.local/share/rmz/trash/"