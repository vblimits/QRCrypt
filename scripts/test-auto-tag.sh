#!/bin/bash
# Test auto-tagging logic (simulates CI behavior)
# Usage: ./scripts/test-auto-tag.sh

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🏷️  Testing Auto-Tagging Logic${NC}"
echo

# Get current version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"

# Check if current version tag exists
CURRENT_TAG="v${CURRENT_VERSION}"
if git rev-parse "$CURRENT_TAG" >/dev/null 2>&1; then
    echo -e "${YELLOW}Tag ${CURRENT_TAG} exists - would increment version${NC}"
    
    # Split version and increment patch
    IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
    NEW_PATCH=$((patch + 1))
    NEW_VERSION="${major}.${minor}.${NEW_PATCH}"
    NEW_TAG="v${NEW_VERSION}"
    
    echo -e "${GREEN}Would create tag: ${NEW_TAG}${NC}"
    echo -e "${BLUE}Would update Cargo.toml to version ${NEW_VERSION}${NC}"
    
else
    echo -e "${YELLOW}Tag ${CURRENT_TAG} does not exist - would use current version${NC}"
    NEW_TAG="$CURRENT_TAG"
    echo -e "${GREEN}Would create tag: ${NEW_TAG}${NC}"
fi

echo
echo "🤖 This simulates the auto-tagging logic from CI:"
echo "   1. Check if current version tag exists"
echo "   2. If yes: increment patch version and tag new version"
echo "   3. If no: tag current version"
echo "   4. Push both commit (if version changed) and tag"
echo
echo -e "${GREEN}✅ Auto-tagging logic test complete!${NC}"