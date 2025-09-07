# Release Process

QRCrypt has two types of releases:

## 🤖 Automated Releases (Recommended)

**How it works:**
- Automatically triggered when CI/CD tests pass on the `main` branch
- Reads version from `Cargo.toml`
- Only creates a release if the version tag doesn't already exist
- Builds Linux binaries (static and dynamic)
- Creates GitHub release with comprehensive changelog

**To create an auto-release:**
1. Update the version in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"  # Increment this
   ```

2. Commit and push to main:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push origin main
   ```

3. Wait for CI to pass - release will be created automatically! 🎉

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