#!/bin/bash
# Version bump script for QRCrypt
# Usage: ./scripts/bump-version.sh [patch|minor|major|X.Y.Z]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo -e "${RED}Error: Could not find version in Cargo.toml${NC}"
    exit 1
fi

echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"

# Parse arguments
BUMP_TYPE=${1:-patch}

# Show help
if [[ "$BUMP_TYPE" == "--help" || "$BUMP_TYPE" == "-h" || "$BUMP_TYPE" == "help" ]]; then
    echo "Usage: $0 [patch|minor|major|X.Y.Z]"
    echo ""
    echo "Examples:"
    echo "  $0 patch          # 0.1.0 → 0.1.1"
    echo "  $0 minor          # 0.1.0 → 0.2.0"
    echo "  $0 major          # 0.1.0 → 1.0.0"
    echo "  $0 1.5.2          # Set specific version"
    echo ""
    echo "Current version: $CURRENT_VERSION"
    exit 0
fi

# Function to increment version
increment_version() {
    local version=$1
    local type=$2
    
    # Split version into parts
    IFS='.' read -r major minor patch <<< "$version"
    
    case $type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            # Assume it's a specific version number
            echo "$type"
            return
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# Calculate new version
if [[ $BUMP_TYPE =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    NEW_VERSION=$BUMP_TYPE
else
    NEW_VERSION=$(increment_version "$CURRENT_VERSION" "$BUMP_TYPE")
fi

echo -e "${GREEN}New version: ${NEW_VERSION}${NC}"

# Confirm with user
read -p "Update version to ${NEW_VERSION}? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Update Cargo.toml
echo "Updating Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml

# Verify the change
UPDATED_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ "$UPDATED_VERSION" != "$NEW_VERSION" ]; then
    echo -e "${RED}Error: Failed to update version${NC}"
    # Restore backup
    mv Cargo.toml.bak Cargo.toml
    exit 1
fi

# Remove backup
rm -f Cargo.toml.bak

echo -e "${GREEN}✅ Version updated to ${NEW_VERSION}${NC}"
echo
echo "Next steps:"
echo "1. Review changes: git diff Cargo.toml"
echo "2. Test build: cargo build --release"
echo "3. Commit changes: git add Cargo.toml && git commit -m 'Bump version to ${NEW_VERSION}'"
echo "4. Push to main: git push origin main"
echo "5. Auto-release will trigger when CI passes! 🚀"
echo
echo -e "${YELLOW}Or create manual cross-platform release:${NC}"
echo "git tag v${NEW_VERSION} && git push origin v${NEW_VERSION}"