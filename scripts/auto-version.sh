#!/bin/bash
# Auto-increment patch version script (simulates CI behavior)
# Usage: ./scripts/auto-version.sh

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get current version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo "Error: Could not find version in Cargo.toml"
    exit 1
fi

echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"

# Split version and increment patch
IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
NEW_PATCH=$((patch + 1))
NEW_VERSION="${major}.${minor}.${NEW_PATCH}"

echo -e "${GREEN}New version: ${NEW_VERSION}${NC}"

# Update Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml

# Verify the change
UPDATED_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ "$UPDATED_VERSION" != "$NEW_VERSION" ]; then
    echo "Error: Failed to update version"
    mv Cargo.toml.bak Cargo.toml
    exit 1
fi

# Remove backup
rm -f Cargo.toml.bak

echo -e "${GREEN}✅ Auto-incremented version to ${NEW_VERSION}${NC}"
echo
echo "🤖 This simulates what happens automatically in CI when you push to main!"
echo "    In CI, this happens after tests pass and before creating releases."