#!/bin/bash

# rmz Alias Installation Script
# This script sets up rmz as a replacement for rm command

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 rmz Alias Setup Script${NC}"
echo "This script will configure rmz as a replacement for the rm command."
echo

# Check if rmz is installed
if ! command -v rmz &> /dev/null; then
    echo -e "${RED}❌ Error: rmz command not found${NC}"
    echo "Please install rmz first or add it to your PATH"
    exit 1
fi

echo -e "${GREEN}✅ rmz found at: $(which rmz)${NC}"
echo

# Detect shell
SHELL_TYPE=$(basename "$SHELL")
case "$SHELL_TYPE" in
    bash)
        CONFIG_FILE="$HOME/.bashrc"
        ;;
    zsh)
        CONFIG_FILE="$HOME/.zshrc"
        ;;
    fish)
        CONFIG_FILE="$HOME/.config/fish/config.fish"
        mkdir -p "$(dirname "$CONFIG_FILE")"
        ;;
    *)
        echo -e "${YELLOW}⚠️  Unknown shell: $SHELL_TYPE${NC}"
        echo "Please manually add aliases to your shell configuration"
        exit 1
        ;;
esac

echo -e "${BLUE}📝 Shell detected: $SHELL_TYPE${NC}"
echo -e "${BLUE}📁 Config file: $CONFIG_FILE${NC}"
echo

# Ask for installation level
echo "Choose alias installation level:"
echo "1) Basic - rm, unrm, trash commands only"
echo "2) Safe - includes interactive mode and safety features"
echo "3) Complete - full replacement with all features"
echo "4) Custom - select specific aliases"
read -p "Enter your choice (1-4): " choice

case $choice in
    1) LEVEL="basic" ;;
    2) LEVEL="safe" ;;
    3) LEVEL="complete" ;;
    4) LEVEL="custom" ;;
    *) echo "Invalid choice"; exit 1 ;;
esac

# Create backup of config file
if [ -f "$CONFIG_FILE" ]; then
    cp "$CONFIG_FILE" "${CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}📦 Backup created: ${CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)${NC}"
fi

# Function to add aliases based on shell type
add_aliases() {
    local aliases="$1"
    
    if [ "$SHELL_TYPE" = "fish" ]; then
        # Convert bash aliases to fish format
        echo "$aliases" | sed 's/alias \([^=]*\)=\(.*\)/alias \1 \2/' >> "$CONFIG_FILE"
    else
        echo "$aliases" >> "$CONFIG_FILE"
    fi
}

# Add rmz alias section header
echo "" >> "$CONFIG_FILE"
echo "# rmz aliases - $(date)" >> "$CONFIG_FILE"
echo "# Backup available at: ${CONFIG_FILE}.backup.*" >> "$CONFIG_FILE"

case $LEVEL in
    "basic")
        ALIASES='
# Basic rmz aliases
alias rm="rmz delete"
alias unrm="rmz restore"
alias trash="rmz list"
alias trash-status="rmz status"
'
        ;;
    "safe")
        ALIASES='
# Safe rmz aliases with interactive mode
alias rm="rmz delete --interactive"
alias rm-force="rmz delete --force"
alias rm-dry="rmz delete --dry-run"
alias unrm="rmz restore --interactive"
alias unrm-all="rmz restore --all"
alias trash="rmz list"
alias trash-status="rmz status"

# Preserve original rm for emergencies
alias rm-original="/bin/rm"
'
        ;;
    "complete")
        ALIASES='
# Complete rmz alias replacement
alias rm="rmz delete --interactive"
alias rm-f="rmz delete --force"
alias rm-rf="rmz delete --force"
alias rm-i="rmz delete --interactive"
alias rm-v="rmz delete --verbose"
alias rm-dry="rmz delete --dry-run"

# Restore commands
alias unrm="rmz restore --interactive"
alias unrm-all="rmz restore --all"
alias unrm-id="rmz restore --id"

# Trash management
alias trash="rmz list"
alias trash-status="rmz status"
alias trash-size="rmz status"

# Tagged operations
alias rm-temp="rmz delete --tag temporary"
alias rm-backup="rmz delete --tag backup"
alias rm-old="rmz delete --tag old-files"

# Safety nets
alias rm-original="/bin/rm"
alias rm-disable="unalias rm && echo \"rmz alias disabled\""
'
        ;;
    "custom")
        echo "Available aliases:"
        echo "1) rm='rmz delete' - Basic replacement"
        echo "2) rm='rmz delete --interactive' - Safe replacement"
        echo "3) unrm='rmz restore' - File restoration"
        echo "4) trash='rmz list' - View deleted files"
        echo "5) trash-status='rmz status' - Trash statistics"
        echo "Enter numbers separated by spaces (e.g., 1 3 4):"
        read -p "Selection: " selections
        
        ALIASES=""
        for sel in $selections; do
            case $sel in
                1) ALIASES="$ALIASES"$'\n'"alias rm=\"rmz delete\"" ;;
                2) ALIASES="$ALIASES"$'\n'"alias rm=\"rmz delete --interactive\"" ;;
                3) ALIASES="$ALIASES"$'\n'"alias unrm=\"rmz restore\"" ;;
                4) ALIASES="$ALIASES"$'\n'"alias trash=\"rmz list\"" ;;
                5) ALIASES="$ALIASES"$'\n'"alias trash-status=\"rmz status\"" ;;
            esac
        done
        ;;
esac

# Add the aliases
add_aliases "$ALIASES"

echo -e "${GREEN}✅ Aliases added to $CONFIG_FILE${NC}"
echo

# Add helper functions for bash/zsh
if [ "$SHELL_TYPE" != "fish" ]; then
    cat >> "$CONFIG_FILE" << 'EOF'

# rmz helper functions
rm_and_status() {
    rmz delete "$@" && rmz status
}
alias rms='rm_and_status'

# Quick restore for last deleted file
unrm_last() {
    local last_id=$(rmz list --limit 1 | grep -o '[a-f0-9]\{8\}' | head -1)
    if [ -n "$last_id" ]; then
        rmz restore --id "$last_id"
    else
        echo "No recently deleted files found"
    fi
}
alias unrm-last='unrm_last'

# More advanced helper functions
trash_cleanup() {
    echo "🗑️  Current trash status:"
    rmz status
    echo ""
    read -p "Empty trash? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Note: Requires purge command implementation
        echo "Purge command not yet implemented"
        # rmz purge --all
    fi
}
alias trash-cleanup='trash_cleanup'

# Show trash and ask for restore
trash_restore() {
    rmz list
    echo ""
    read -p "Enter file ID to restore (or press Enter to skip): " file_id
    if [ -n "$file_id" ]; then
        rmz restore --id "$file_id"
    fi
}
alias trash-restore='trash_restore'
EOF
    echo -e "${GREEN}✅ Helper functions added${NC}"
fi

# Instructions
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. Reload your shell configuration:"
case $SHELL_TYPE in
    bash|zsh) echo "   source $CONFIG_FILE" ;;
    fish) echo "   source $CONFIG_FILE" ;;
esac
echo "2. Or restart your terminal"
echo "3. Test the aliases:"
echo "   rm --help    # Should show rmz delete help"
echo "   trash        # Should show rmz list"
echo

# Safety reminder
echo -e "${RED}⚠️  Important Safety Notes:${NC}"
echo "• The original rm command is still available as 'rm-original'"
echo "• Use 'rm-dry' for dry-run before actual deletion"
echo "• Your original config is backed up with timestamp"
echo "• To disable rmz aliases: rm-disable"
echo

echo -e "${GREEN}🎉 rmz alias setup completed!${NC}"
echo "You can now use rm commands with the safety of rmz."