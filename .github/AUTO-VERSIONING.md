# 🤖 Auto-Versioning System

QRCrypt now automatically manages patch versions! Every time you push to the main branch and tests pass, the system automatically increments the patch version.

## How It Works

### 🔄 Automatic Process
1. **You push code** to main branch
2. **CI runs tests** (test, security, build)
3. **If tests pass**, CI automatically:
   - Increments patch version (0.96.0 → 0.96.1)
   - **Creates git tag** (v0.96.1) 
   - Builds release binaries  
   - Creates GitHub release
   - Commits new version back to repo
   - **Pushes tag to GitHub**
4. **Next push** will increment again (0.96.1 → 0.96.2)

### 📋 Version Logic
- **First push**: If no tag exists for current version, uses current version
- **Subsequent pushes**: Always increments patch version
- **Smart tagging**: Won't create duplicate releases

## Usage Examples

### 🚀 Normal Development (Automatic)
```bash
# Just push your changes - versioning and tagging is automatic!
git add .
git commit -m "Fix bug in QR generation"
git push origin main
# → CI will create tag v0.96.1 AND release v0.96.1 automatically
```

### 📈 Major/Minor Version Bumps (Manual)
```bash
# For new features (minor version)
./scripts/bump-version.sh minor  # 0.96.x → 0.97.0
git add Cargo.toml
git commit -m "Bump to v0.97.0 for new features"
git push origin main
# → CI will create v0.97.0, then subsequent pushes become v0.97.1, v0.97.2...

# For breaking changes (major version)  
./scripts/bump-version.sh major  # 0.96.x → 1.0.0
git add Cargo.toml
git commit -m "Bump to v1.0.0 - breaking changes"
git push origin main
# → CI will create v1.0.0, then subsequent pushes become v1.0.1, v1.0.2...
```

## Benefits

✅ **Zero manual work** - just push code  
✅ **Automatic git tags** - no manual tagging required  
✅ **Consistent versioning** - no forgotten version bumps  
✅ **Automatic releases** - users get updates immediately  
✅ **Safe releases** - only happens when tests pass  
✅ **Version tracking** - all changes are committed back  
✅ **Tag synchronization** - tags and releases always match  

## Files Changed
- `.github/workflows/ci.yml` - Added auto-versioning logic
- `scripts/auto-version.sh` - Local testing script  
- `scripts/bump-version.sh` - Enhanced for major/minor bumps
- Documentation updated with new workflow

## Current State
- **Version**: 0.96.0 (ready for first auto-increment)
- **Next release**: Will be v0.96.1 when you push to main
- **Pattern**: Each push increments: 0.96.1 → 0.96.2 → 0.96.3 → ...

---

🎉 **You're all set!** Just push your changes and watch the magic happen!