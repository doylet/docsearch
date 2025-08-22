#!/bin/bash
# Create DMG package for Zero-Latency

set -e

APP_NAME="Zero-Latency"
VERSION="1.0.0"
DMG_NAME="Zero-Latency-v${VERSION}"
BUNDLE_DIR="$APP_NAME.app"

echo "🚀 Creating DMG package..."

# Build the app bundle first
./scripts/build-macos-app.sh

# Check if app bundle exists
if [ ! -d "$BUNDLE_DIR" ]; then
    echo "❌ App bundle not found. Build failed."
    exit 1
fi

# Create temporary DMG staging directory
STAGING_DIR="dmg-staging"
rm -rf "$STAGING_DIR"
mkdir "$STAGING_DIR"

# Copy app bundle to staging
cp -R "$BUNDLE_DIR" "$STAGING_DIR/"

# Create README for DMG
cat > "$STAGING_DIR/README.txt" << 'EOF'
Zero-Latency Documentation Search v1.0.0

INSTALLATION:
1. Drag Zero-Latency.app to your Applications folder
2. Double-click Zero-Latency.app to open the control panel
3. Choose "Install & Start Daemon" to begin

USAGE:
- The daemon runs in the background on port 8080
- Use the CLI tool: /Applications/Zero-Latency.app/Contents/MacOS/mdx
- Or open a CLI terminal from the control panel

FEATURES:
✅ Semantic document search with local embeddings
✅ REST API, JSON-RPC 2.0, and MCP protocol support
✅ HTTP streaming and stdio transport
✅ Clean architecture with dependency injection

For support: https://github.com/your-repo/zero-latency
EOF

# Create simple installer script (optional)
cat > "$STAGING_DIR/Install.command" << 'EOF'
#!/bin/bash
echo "🚀 Installing Zero-Latency..."

# Copy to Applications if not already there
if [ ! -d "/Applications/Zero-Latency.app" ]; then
    echo "📦 Copying to Applications..."
    cp -R "Zero-Latency.app" "/Applications/"
    echo "✅ Installed to /Applications/Zero-Latency.app"
else
    echo "ℹ️  Already installed in Applications"
fi

echo "🎉 Installation complete!"
echo "💡 Launch from Applications or run: open /Applications/Zero-Latency.app"

read -p "Press Enter to continue..."
EOF

chmod +x "$STAGING_DIR/Install.command"

# Calculate size for DMG
SIZE=$(du -sm "$STAGING_DIR" | cut -f1)
SIZE=$((SIZE + 50))  # Add some padding

# Create DMG
echo "📦 Creating DMG (${SIZE}MB)..."
hdiutil create -srcfolder "$STAGING_DIR" \
    -volname "$APP_NAME" \
    -fs HFS+ \
    -fsargs "-c c=64,a=16,e=16" \
    -format UDRW \
    -size "${SIZE}m" \
    "temp-${DMG_NAME}.dmg"

# Mount the DMG for customization
echo "🎨 Customizing DMG..."
MOUNT_DIR="/Volumes/$APP_NAME"
hdiutil attach "temp-${DMG_NAME}.dmg"

# Wait for mount
sleep 2

# Create Applications alias
ln -sf /Applications "$MOUNT_DIR/Applications"

# Set custom DMG view (if desired)
osascript << 'EOD'
tell application "Finder"
    tell disk "Zero-Latency"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {400, 100, 900, 400}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 72
        set position of item "Zero-Latency.app" of container window to {150, 150}
        set position of item "Applications" of container window to {350, 150}
        set position of item "README.txt" of container window to {250, 250}
        close
    end tell
end tell
EOD

# Unmount and convert to read-only
hdiutil detach "$MOUNT_DIR"
hdiutil convert "temp-${DMG_NAME}.dmg" -format UDZO -o "${DMG_NAME}.dmg"

# Clean up
rm "temp-${DMG_NAME}.dmg"
rm -rf "$STAGING_DIR"

echo "✅ DMG created: ${DMG_NAME}.dmg"
echo "📏 Size: $(du -h "${DMG_NAME}.dmg" | cut -f1)"
echo "🎉 Ready for distribution!"
