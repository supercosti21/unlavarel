# CI/CD Pipeline

## Overview

Unlavarel uses GitHub Actions to build release artifacts for all three platforms. The pipeline is defined in `.github/workflows/build.yml`.

## Build Matrix

| Runner | Platform | Output | Notes |
|--------|----------|--------|-------|
| `macos-latest` | macOS | `.dmg` (universal) | Intel + Apple Silicon |
| `ubuntu-22.04` | Linux | `.AppImage`, `.deb` | Requires webkit2gtk-4.1 |
| `windows-latest` | Windows | `.exe`, `.msi` | WebView2 bundled |

## Pipeline Steps

```yaml
1. Checkout code
2. Install Rust stable toolchain
   - macOS: add aarch64-apple-darwin + x86_64-apple-darwin targets
3. Install platform dependencies
   - Ubuntu: libwebkit2gtk-4.1-dev, libappindicator3-dev, patchelf
4. Setup Node.js 22 with npm cache
5. npm ci (install frontend dependencies)
6. Build with tauri-apps/tauri-action@v0
```

## Triggers

The pipeline runs on:
- **Push to `main`** — Builds but no release
- **Pull requests to `main`** — Build validation
- **Tag push (`v*`)** — Builds and creates a GitHub draft release

## Release Process

1. Update version in `package.json`, `Cargo.toml`, and `tauri.conf.json`
2. Commit the version bump
3. Create a git tag: `git tag v0.1.0`
4. Push with tag: `git push origin main --tags`
5. CI builds all three platforms
6. A draft release is created with all artifacts
7. Review and publish the release

## macOS Code Signing

The current build is **unsigned**. For personal use, users run:
```bash
xattr -rd com.apple.quarantine /Applications/Unlavarel.app
```

For future releases, Apple Developer ID signing can be added by setting:
- `APPLE_CERTIFICATE` — Base64-encoded .p12 certificate
- `APPLE_CERTIFICATE_PASSWORD` — Certificate password
- `APPLE_SIGNING_IDENTITY` — Developer ID identity
- `APPLE_ID` — Apple ID for notarization
- `APPLE_PASSWORD` — App-specific password

## Linux Packaging

The `tauri-action` produces:
- **AppImage** — Universal, works on any distro
- **deb** — For Debian/Ubuntu (installable via `sudo dpkg -i`)

Future additions:
- **RPM** — For Fedora/RHEL
- **Flatpak** — For sandboxed distribution
- **AUR** — For Arch Linux

## Windows Packaging

- **MSI installer** — Standard Windows installer
- **EXE** — NSIS-based installer

WebView2 runtime is bundled for Windows 10 compatibility.

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `GITHUB_TOKEN` | Required for release creation |
| `TAURI_PRIVATE_KEY` | For update signature (optional) |
| `TAURI_KEY_PASSWORD` | Update key password (optional) |
