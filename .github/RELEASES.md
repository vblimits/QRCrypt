# Release Process

QRCrypt has automatic version management and releases:

## 🤖 Automated Versioning & Releases

**How it works:**
- **Auto-versioning**: When you push to `main`, CI automatically increments the patch version (0.96.0 → 0.96.1)
- **Smart releases**: Only creates releases if the new version tag doesn't exist
- **Builds & publishes**: Creates Linux binaries and comprehensive GitHub releases
- **Version tracking**: Commits the new version back to the repo

**Simple workflow:**
1. Make your changes and commit:
   ```bash
   git add .
   git commit -m "Add new feature"
   git push origin main
   ```

2. CI automatically:
   - ✅ Runs tests
   - 🔢 Increments patch version (0.96.0 → 0.96.1)
   - 📦 Creates release with new version
   - 💾 Commits version bump back to repo

3. Next push will auto-increment again (0.96.1 → 0.96.2) 🔄

**Manual version control:**
- **Major/Minor bumps**: Use `./scripts/bump-version.sh major` or `minor`
- **Patch auto-increment**: Happens automatically on every push to main

## 🛠️ Manual Cross-Platform Releases

**How it works:**
- Triggered by pushing tags or manual workflow dispatch
- Builds for multiple platforms (Linux, Windows, macOS)
- Creates comprehensive release with all platform binaries

**To create a manual release:**
1. Create and push a tag:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. Or use GitHub Actions workflow dispatch with tag input

## 📋 Release Checklist

Before releasing:
- [ ] All tests pass locally: `cargo test`
- [ ] Code is properly formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Version updated in `Cargo.toml`
- [ ] CHANGELOG or release notes updated (if needed)
- [ ] Security audit clean: `cargo audit`

## 🔄 Version Bump Guidelines

- **Patch** (0.1.0 → 0.1.1): Bug fixes, security updates
- **Minor** (0.1.0 → 0.2.0): New features, non-breaking changes  
- **Major** (0.1.0 → 1.0.0): Breaking changes, major overhauls

## 🛡️ Security Releases

For security-critical updates:
1. Update dependencies with `cargo update`
2. Run security audit: `cargo audit`
3. Bump version (patch for security fixes)
4. Push to main - auto-release will handle the rest

## 📚 Release Assets

**Auto-releases include:**
- `qrcrypt-linux-x86_64.tar.gz` - Dynamic Linux binary
- `qrcrypt-linux-x86_64-static.tar.gz` - Static Linux binary (recommended)

**Manual releases include:**
- Linux (x86_64, x86_64-musl)
- Windows (x86_64)  
- macOS (x86_64, aarch64)

---

*The auto-release system ensures every passing commit on main can potentially become a release, maintaining high code quality and rapid iteration.*