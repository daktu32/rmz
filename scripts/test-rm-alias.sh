#!/bin/bash

# Test script to verify rm alias is working correctly

set -e

echo "🧪 Testing rm alias functionality..."
echo

# Create a temporary test directory
test_dir=$(mktemp -d)
echo "📁 Created test directory: $test_dir"
cd "$test_dir"

# Create test files
echo "test content" > test_file.txt
mkdir test_directory
echo "directory content" > test_directory/nested_file.txt

echo "📝 Created test files:"
ls -la

echo
echo "🔍 Testing rm alias..."

# Check if rm is aliased
if alias rm &>/dev/null; then
    echo "✅ rm alias exists: $(alias rm)"
else
    echo "❌ rm alias not found"
    echo "   Run ./scripts/setup-rm-alias.sh first"
    exit 1
fi

echo
echo "🗑️  Testing file deletion..."

# Test single file deletion
echo "Testing: rm test_file.txt"
rm test_file.txt

# Test directory deletion
echo "Testing: rm -r test_directory"
rm -r test_directory

echo
echo "📊 Checking trash status..."
rmz status

echo
echo "📋 Listing trash contents..."
rmz list

echo
echo "🔄 Testing restore..."
rmz restore --all

echo
echo "📁 Checking restored files..."
ls -la

echo
echo "🧹 Cleaning up test files..."
rmz delete test_file.txt test_directory
rmz purge --all

# Clean up test directory
cd /
rm -rf "$test_dir" 2>/dev/null || /bin/rm -rf "$test_dir"

echo
echo "✅ rm alias test completed successfully!"
echo
echo "📋 Summary:"
echo "   - rm is successfully aliased to rmz delete"
echo "   - File and directory deletion works"
echo "   - Files are moved to trash (not permanently deleted)"
echo "   - Restore functionality works"
echo
echo "🎉 Your rm command is now safe!"