#!/bin/bash
# Package Zero-Latency as macOS App Bundle

set -e

APP_NAME="Zero-Latency"
BUNDLE_DIR="$APP_NAME.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
LAUNCH_AGENTS_DIR="$RESOURCES_DIR/LaunchAgents"

echo "🚀 Building macOS App Bundle for $APP_NAME"

# Clean up previous build
rm -rf "$BUNDLE_DIR"

# Create directory structure
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"
mkdir -p "$LAUNCH_AGENTS_DIR"

# Copy binaries
echo "📦 Copying binaries..."
cp target/release/doc-indexer "$MACOS_DIR/"
cp target/release/mdx "$MACOS_DIR/"

# Create Info.plist
cat > "$CONTENTS_DIR/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>Zero-Latency</string>
    <key>CFBundleIdentifier</key>
    <string>com.zerolatency.app</string>
    <key>CFBundleName</key>
    <string>Zero-Latency</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSUIElement</key>
    <true/>
    <key>LSBackgroundOnly</key>
    <false/>
</dict>
</plist>
EOF

# Create main app launcher script
cat > "$MACOS_DIR/Zero-Latency" << 'EOF'
#!/bin/bash
# Zero-Latency App Launcher

APP_DIR="$(dirname "$0")"
RESOURCES_DIR="$APP_DIR/../Resources"
LAUNCH_AGENT_PLIST="com.zerolatency.doc-indexer.plist"
USER_LAUNCH_AGENTS="$HOME/Library/LaunchAgents"

# Function to install launch agent
install_daemon() {
    echo "🔧 Installing Zero-Latency daemon..."
    
    # Create user launch agents directory if it doesn't exist
    mkdir -p "$USER_LAUNCH_AGENTS"
    
    # Copy launch agent plist
    cp "$RESOURCES_DIR/LaunchAgents/$LAUNCH_AGENT_PLIST" "$USER_LAUNCH_AGENTS/"
    
    # Update paths in plist to point to actual binary location
    sed -i '' "s|{{APP_DIR}}|$APP_DIR|g" "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST"
    
    # Load the launch agent
    launchctl load "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST"
    
    echo "✅ Daemon installed and started"
}

# Function to uninstall launch agent
uninstall_daemon() {
    echo "🗑️  Uninstalling Zero-Latency daemon..."
    
    # Unload and remove launch agent
    launchctl unload "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    rm -f "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST"
    
    echo "✅ Daemon uninstalled"
}

# Function to show status
show_status() {
    if launchctl list | grep -q "com.zerolatency.doc-indexer"; then
        echo "✅ Zero-Latency daemon is running"
        echo "🌐 Service available at: http://localhost:8080"
        echo "🔧 CLI tool: $APP_DIR/mdx"
    else
        echo "❌ Zero-Latency daemon is not running"
    fi
}

# Function to open CLI
open_cli() {
    echo "🖥️  Opening Zero-Latency CLI..."
    osascript << EOD
tell application "Terminal"
    do script "cd '$APP_DIR' && echo 'Zero-Latency CLI Ready! Try: ./mdx search \"your query\"' && bash"
    activate
end tell
EOD
}

# Main menu
echo "🚀 Zero-Latency Documentation Search"
echo ""
echo "1) Install & Start Daemon"
echo "2) Stop & Uninstall Daemon" 
echo "3) Show Status"
echo "4) Open CLI Terminal"
echo "5) Exit"
echo ""
read -p "Choose an option (1-5): " choice

case $choice in
    1) install_daemon ;;
    2) uninstall_daemon ;;
    3) show_status ;;
    4) open_cli ;;
    5) exit 0 ;;
    *) echo "Invalid option" ;;
esac
EOF

# Make launcher executable
chmod +x "$MACOS_DIR/Zero-Latency"

# Create LaunchAgent plist
cat > "$LAUNCH_AGENTS_DIR/com.zerolatency.doc-indexer.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.zerolatency.doc-indexer</string>
    <key>ProgramArguments</key>
    <array>
        <string>{{APP_DIR}}/doc-indexer</string>
        <string>--port</string>
        <string>8080</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/zero-latency.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/zero-latency-error.log</string>
    <key>WorkingDirectory</key>
    <string>{{APP_DIR}}</string>
</dict>
</plist>
EOF

echo "✅ App bundle created: $BUNDLE_DIR"
echo "🚀 Double-click $BUNDLE_DIR to run!"
