# Build and Distribution Scripts

This directory contains automated scripts for building and packaging Zero-Latency for distribution.

## Scripts Overview

### `build-macos-app.sh`

Creates a professional macOS application bundle with integrated daemon management.

**Usage:**
```bash
./scripts/build-macos-app.sh
```

**Output:**
- `Zero-Latency.app` - Complete macOS app bundle
- Includes both `doc-indexer` and `mdx` binaries
- LaunchAgent configuration for background daemon
- GUI control panel for user interaction

**Features:**
- Self-contained application structure
- Automatic daemon installation and management
- Integrated CLI terminal access
- Professional macOS app bundle format

### `build-dmg.sh`

Packages the app bundle into a distributable DMG installer.

**Usage:**
```bash
./scripts/build-dmg.sh
```

**Prerequisites:**
- Must run `build-macos-app.sh` first (or will auto-run)
- Requires `hdiutil` (standard on macOS)

**Output:**
- `Zero-Latency-v1.0.0.dmg` - Professional DMG installer
- Includes Applications folder alias
- Comprehensive README for end users
- Drag-and-drop installation experience

### `install.sh`

Command-line installation script for direct binary deployment.

**Usage:**
```bash
./scripts/install.sh
```

**Features:**
- Installs binaries to `/usr/local/bin`
- Creates LaunchAgent for background daemon
- Automatic service management
- Command-line focused installation

## Build Prerequisites

### Required Tools

- **Rust toolchain**: `cargo build --release` must work
- **macOS**: Scripts designed for macOS distribution
- **hdiutil**: For DMG creation (standard on macOS)

### Build Dependencies

Ensure release binaries are built before running packaging scripts:

```bash
# Build release binaries first
cargo build --release

# Then run packaging scripts
./scripts/build-macos-app.sh
./scripts/build-dmg.sh
```

## Build Process

### Complete Distribution Build

```bash
# 1. Build release binaries
cargo build --release

# 2. Create macOS app bundle
./scripts/build-macos-app.sh

# 3. Package into DMG
./scripts/build-dmg.sh

# Results:
# - target/release/{doc-indexer,mdx} (binaries)
# - Zero-Latency.app (app bundle)
# - Zero-Latency-v1.0.0.dmg (installer)
```

### Individual Steps

```bash
# Just create app bundle
./scripts/build-macos-app.sh

# Just create DMG (requires existing app bundle)
./scripts/build-dmg.sh

# Command-line installation only
./scripts/install.sh
```

## Distribution Files

### App Bundle Structure

```
Zero-Latency.app/
├── Contents/
│   ├── Info.plist              # App metadata
│   ├── MacOS/
│   │   ├── Zero-Latency        # Main GUI launcher
│   │   ├── doc-indexer         # API server binary
│   │   └── mdx                 # CLI binary
│   └── Resources/
│       └── LaunchAgents/
│           └── com.zerolatency.doc-indexer.plist
```

### DMG Contents

```
Zero-Latency-v1.0.0.dmg
├── Zero-Latency.app            # Main application
├── Applications (alias)        # For drag-and-drop
└── README.txt                  # Installation instructions
```

## Configuration

### Version Management

Update version number in `build-dmg.sh`:

```bash
VERSION="1.0.0"
DMG_NAME="Zero-Latency-v${VERSION}"
```

### App Bundle Settings

Modify app metadata in `build-macos-app.sh`:

```xml
<key>CFBundleVersion</key>
<string>1.0</string>
<key>CFBundleIdentifier</key>
<string>com.zerolatency.app</string>
```

### LaunchAgent Configuration

Daemon settings configured in both scripts:

```xml
<key>Label</key>
<string>com.zerolatency.doc-indexer</string>
<key>ProgramArguments</key>
<array>
    <string>/Applications/docsearch.app/Contents/MacOS/doc-indexer</string>
    <string>--port</string>
    <string>8080</string>
</array>
```

## Customization

### Adding Features

To add new features to the app bundle:

1. **Modify `build-macos-app.sh`**:
   - Add new binaries to `MacOS/` directory
   - Update Info.plist if needed
   - Add additional resources

2. **Update GUI launcher**:
   - Modify the main app executable
   - Add new menu options or functionality

3. **Extend LaunchAgent**:
   - Add environment variables
   - Configure additional services

### Distribution Channels

Scripts can be adapted for different distribution methods:

- **Mac App Store**: Add code signing and notarization
- **Homebrew**: Create formula using install.sh
- **Direct Download**: Use DMG as-is

## Troubleshooting

### Build Failures

**"cargo build failed"**:
```bash
# Ensure Rust toolchain is installed
rustup update
cargo clean
cargo build --release
```

**"App bundle creation failed"**:
```bash
# Check binary paths
ls -la target/release/{doc-indexer,mdx}
# Ensure binaries are executable
chmod +x target/release/{doc-indexer,mdx}
```

**"DMG creation failed"**:
```bash
# Check app bundle exists
ls -la Zero-Latency.app
# Check available disk space
df -h .
```

### Runtime Issues

**"Daemon won't start"**:
```bash
# Check LaunchAgent syntax
plutil -lint ~/Library/LaunchAgents/com.zerolatency.doc-indexer.plist
# Manual daemon test
/Applications/docsearch.app/Contents/MacOS/doc-indexer --help
```

**"CLI not found"**:
```bash
# Check installation path
ls -la /usr/local/bin/{doc-indexer,mdx}
# Check PATH
echo $PATH
```

## Security Considerations

### Code Signing

For distribution outside development:

```bash
# Sign app bundle
codesign --force --sign "Developer ID Application: Your Name" Zero-Latency.app

# Sign DMG
codesign --force --sign "Developer ID Application: Your Name" Zero-Latency-v1.0.0.dmg
```

### Notarization

For Gatekeeper compatibility:

```bash
# Notarize app
xcrun notarytool submit Zero-Latency-v1.0.0.dmg --keychain-profile "notarytool-profile"

# Staple notarization
xcrun stapler staple Zero-Latency-v1.0.0.dmg
```

## Maintenance

### Regular Updates

1. **Version Bump**: Update version numbers in scripts
2. **Test Build**: Run complete build process
3. **Quality Check**: Test installation and functionality
4. **Documentation**: Update README and release notes

### Cleanup

```bash
# Remove build artifacts
rm -rf Zero-Latency.app
rm -f Zero-Latency-v*.dmg

# Clean cargo build
cargo clean
```
