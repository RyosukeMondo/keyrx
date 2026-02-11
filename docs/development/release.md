# Release Process

This document describes how to create a new release of KeyRx.

## Prerequisites

- GitHub CLI (`gh`) authenticated: `gh auth login`
- All tests passing: `make verify`
- Clean working directory: `git status`
- Changelog updated with new version

## Release Steps

### 1. Update Version

Update version in `Cargo.toml`:
```toml
[workspace.package]
version = "0.2.0"  # Update this
```

### 2. Update Changelog

Add release notes to `CHANGELOG.md` (create if doesn't exist):
```markdown
# Changelog

## [0.2.0] - 2024-01-25

### Added
- Linux installers (.deb and .tar.gz)
- Automated GitHub releases
- Systemd service integration

### Fixed
- Bug fixes...

### Changed
- Improvements...
```

### 3. Commit Version Bump

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push
```

### 4. Create and Push Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag (this triggers the release workflow)
git push origin v0.2.0
```

### 5. Monitor Release Build

The GitHub Actions workflow will automatically:
1. Build binaries for Linux and Windows
2. Build .deb package
3. Build .tar.gz with install script
4. Create GitHub release with all artifacts
5. Generate installation instructions

Monitor progress at:
https://github.com/RyosukeMondo/keyrx/actions

### 6. Verify Release

Once the workflow completes (5-10 minutes):

1. Check the release page: https://github.com/RyosukeMondo/keyrx/releases
2. Verify all files are attached:
   - `keyrx_0.2.0_amd64.deb`
   - `keyrx-0.2.0-linux-x86_64.tar.gz`
   - `keyrx_compiler-linux-x86_64`
   - `keyrx_daemon-linux-x86_64`
   - Windows binaries
3. Test installation on a clean system

### 7. Announce Release

- Update project README if needed
- Post announcement (social media, forums, etc.)
- Update documentation website if applicable

## Testing Packages Locally

Before creating a release, test package building locally:

```bash
# Build all packages
./scripts/package/build-all.sh

# Test Debian package
sudo dpkg -i target/debian/keyrx_0.2.0_amd64.deb
sudo systemctl start keyrx
curl http://localhost:7777

# Test tarball
tar -xzf target/tarball/keyrx-0.2.0-linux-x86_64.tar.gz
cd keyrx-0.2.0-linux-x86_64
sudo ./install.sh
```

## Rollback a Release

If you need to delete a release:

```bash
# Delete the release
gh release delete v0.2.0 --yes

# Delete the tag locally and remotely
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0
```

## Troubleshooting

### Workflow Fails

1. Check the Actions log: https://github.com/RyosukeMondo/keyrx/actions
2. Common issues:
   - Build failures: Fix code and create new tag
   - Missing dependencies: Update workflow
   - Permission errors: Check GITHUB_TOKEN permissions

### Package Build Fails Locally

```bash
# Clean build artifacts
cargo clean
rm -rf target/debian target/tarball

# Rebuild from scratch
./scripts/package/build-all.sh
```

### Installation Issues

**Debian package:**
```bash
# Check for conflicts
dpkg -l | grep keyrx

# Force reinstall
sudo dpkg -r keyrx
sudo dpkg -i keyrx_0.2.0_amd64.deb
```

**Tarball:**
```bash
# Uninstall first
sudo ./uninstall.sh

# Reinstall
sudo ./install.sh
```

## Release Checklist

- [ ] All tests passing (`make verify`)
- [ ] Version updated in Cargo.toml
- [ ] Changelog updated
- [ ] Changes committed and pushed
- [ ] Tag created and pushed
- [ ] Workflow completed successfully
- [ ] Release artifacts verified
- [ ] Installation tested on clean system
- [ ] Documentation updated
- [ ] Release announced

## Quick Release (One-liner)

For experienced maintainers:

```bash
# Update version in Cargo.toml first, then:
VERSION=0.2.0 && \
git add Cargo.toml CHANGELOG.md && \
git commit -m "chore: bump version to $VERSION" && \
git tag -a "v$VERSION" -m "Release v$VERSION" && \
git push && git push origin "v$VERSION"
```

Monitor at: https://github.com/RyosukeMondo/keyrx/actions
