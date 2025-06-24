#!/bin/bash

set -e

echo "🐋 Building rmz container..."
docker-compose build

echo "🚀 Starting rmz test environment..."
docker-compose run --rm rmz-test sh -c "
echo '=== rmz Testing Environment ==='
echo
echo '🏠 Current directory: '\$(pwd)
echo '📁 Available test files:'
find test -type f -exec ls -la {} \;
echo
echo '🔧 rmz version:'
rmz --version
echo
echo '📋 Available commands:'
rmz --help
echo
echo '🎯 Quick test - delete a file:'
rmz delete test/temp/cache.tmp
echo
echo '📊 Check status:'
rmz status
echo
echo '📝 List deleted files:'
rmz list
echo
echo '🔄 Restore the file:'
rmz restore -i 1
echo
echo '✅ Test completed! You can now explore rmz interactively.'
echo
echo 'Starting interactive shell...'
exec sh
"