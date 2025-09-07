# 🏷️ Auto-Tagging System

QRCrypt now automatically creates and manages Git tags! No more manual tagging - just push your code and CI handles everything.

## How It Works

### 🤖 Automatic Git Tagging
When you push to `main` and tests pass, CI automatically:

1. **Checks current version** in Cargo.toml (e.g., 0.96.0)
2. **Checks if tag exists** for that version (v0.96.0)
3. **Smart versioning**:
   - If tag exists → increment patch (0.96.0 → 0.96.1) 
   - If tag doesn't exist → use current version (0.96.0)
4. **Creates annotated tag** with release message
5. **Pushes tag to GitHub** automatically
6. **Creates release** using the new tag

## Workflow Examples

### 📝 First Push (No Existing Tag)
```bash
# Current Cargo.toml version: 0.96.0
git add .
git commit -m "Initial release"
git push origin main

# CI Results:
# ✅ Tests pass
# 🏷️ Creates tag: v0.96.0 (current version)
# 📦 Creates release: v0.96.0
```

### 🔄 Subsequent Pushes (Tag Exists)
```bash
# Current Cargo.toml version: 0.96.0 (tag v0.96.0 exists)
git add .
git commit -m "Bug fix"
git push origin main

# CI Results:
# ✅ Tests pass
# 🔢 Updates Cargo.toml: 0.96.0 → 0.96.1
# 🏷️ Creates tag: v0.96.1
# 📦 Creates release: v0.96.1
# 💾 Commits version bump back to repo
```

### 📈 Next Push (Continues Auto-Incrementing)
```bash
# Current Cargo.toml version: 0.96.1 (tag v0.96.1 exists)
git add .
git commit -m "New feature"
git push origin main

# CI Results:
# ✅ Tests pass
# 🔢 Updates Cargo.toml: 0.96.1 → 0.96.2
# 🏷️ Creates tag: v0.96.2
# 📦 Creates release: v0.96.2
# 💾 Commits version bump back to repo
```

## Tag Features

### 🏷️ Tag Format
- **Name**: `vX.Y.Z` (e.g., v0.96.1, v1.0.0)
- **Type**: Annotated tags (not lightweight)
- **Message**: `🤖 Auto-release vX.Y.Z - All tests passed`

### 📋 Tag Management
- **Automatic creation**: No manual `git tag` needed
- **Automatic pushing**: Tags are pushed with `--tags` flag
- **No duplicates**: Won't create tags that already exist
- **Synchronized releases**: Every tag gets a matching GitHub release

## Benefits

✅ **Zero manual tagging** - completely automated  
✅ **Perfect synchronization** - tags and releases always match  
✅ **Consistent naming** - vX.Y.Z format always  
✅ **Safe tagging** - only happens when tests pass  
✅ **Annotated tags** - include commit messages and metadata  
✅ **Version tracking** - easy to see release history via `git tag`  

## Manual Override

If you need manual control (rare cases):

```bash
# Use the manual cross-platform release workflow
# Go to GitHub Actions → "Manual Cross-Platform Release"
# Set tag name and check "create_tag" if needed
```

## Testing Locally

```bash
# Test the auto-tagging logic
./scripts/test-auto-tag.sh

# Test auto-versioning
./scripts/auto-version.sh
```

---

🎉 **You're set!** Just push to main - tags and releases happen automatically!