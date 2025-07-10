# GitHub Actions Workflows

This directory contains CI/CD workflows for the GLSP project.

## Workflows

### 1. CI (`ci.yml`)
- **Triggers**: Push to main/develop, Pull requests
- **Actions**:
  - Run tests on Linux, Windows, and macOS
  - Check code formatting with rustfmt
  - Run clippy lints
  - Build web client
  - Test Tauri build (debug mode for speed)

### 2. Release Build (`release.yml`)
- **Triggers**: Push tags starting with `v*` (e.g., `v1.0.0`)
- **Actions**:
  - Creates a draft GitHub release
  - Builds for all platforms:
    - macOS ARM64 (Apple Silicon)
    - macOS x64 (Intel)
    - Linux x64 (.AppImage and .deb)
    - Windows x64 (.msi)
  - Uploads artifacts to the release
  - Publishes the release when all builds complete

### 3. Universal macOS Binary (`build-universal-mac.yml`)
- **Triggers**: Manual workflow dispatch or tags ending with `-universal`
- **Actions**:
  - Builds both ARM64 and x64 macOS binaries
  - Creates a universal binary using `lipo`
  - Uploads the universal binary as an artifact

## Creating a Release

1. **Update version** in:
   - `glsp-tauri/src-tauri/tauri.conf.json`
   - `glsp-tauri/src-tauri/Cargo.toml`
   - `glsp-tauri/package.json`

2. **Create and push a tag**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

3. **Monitor the Actions tab** on GitHub to see the build progress

4. **The release will be created automatically** with all platform binaries

## Platform-Specific Notes

### macOS
- Builds both ARM64 and x64 versions
- Creates .dmg installers
- No code signing configured (add your certificate in secrets)

### Linux
- Builds on Ubuntu 20.04 for compatibility
- Creates .AppImage (portable) and .deb (Debian/Ubuntu) packages
- Requires GTK3 and WebKit2GTK

### Windows
- Builds on latest Windows
- Creates .msi installer
- No code signing configured (add certificate in workflow)

## Secrets Required

For production releases, you may want to add these secrets:
- `APPLE_CERTIFICATE`: Base64 encoded .p12 certificate
- `APPLE_CERTIFICATE_PASSWORD`: Certificate password
- `APPLE_TEAM_ID`: Your Apple Developer Team ID
- `WINDOWS_CERTIFICATE`: Base64 encoded .pfx certificate
- `WINDOWS_CERTIFICATE_PASSWORD`: Certificate password

## Manual Builds

To trigger a manual build without creating a tag:
1. Go to Actions tab
2. Select "Release Build" workflow
3. Click "Run workflow"
4. Enter a version tag (e.g., `v0.0.0-test`)

## Local Testing

To test workflows locally, use [act](https://github.com/nektos/act):
```bash
act -j test
```