#!/bin/bash
# Package Zero-Latency as macOS App Bundle

set -e

APP_NAME="Zero-Latency"
BUNDLE_DIR="./dist/$APP_NAME.app"
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

# Copy documentation directory
echo "📚 Copying documentation..."
cp -r docs "$RESOURCES_DIR/"

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
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSHighResolutionCapable</key>
    <true/>
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

# Check if running in Terminal (has TTY) or launched from Finder
if [ ! -t 1 ] && [ ! -t 0 ]; then
    # Launched from Finder - open in Terminal
    osascript << EOD
tell application "Terminal"
    do script "cd '$APP_DIR' && ./Zero-Latency"
    activate
end tell
EOD
    exit 0
fi

# Function to install launch agent
install_daemon() {
    echo "🔧 Installing Zero-Latency daemon..."
    
    # Create user launch agents directory if it doesn't exist
    mkdir -p "$USER_LAUNCH_AGENTS"
    
    # Copy launch agent plist
    cp "$RESOURCES_DIR/LaunchAgents/$LAUNCH_AGENT_PLIST" "$USER_LAUNCH_AGENTS/"
    
    # Update paths in plist to point to actual binary location
    sed -i '' "s|{{APP_DIR}}|$APP_DIR|g" "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST"
    
    # Unload any existing service first
    launchctl unload "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    
    # Load the launch agent
    echo "📡 Loading launch agent..."
    if launchctl load "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST" 2>/dev/null; then
        echo "✅ Launch agent loaded successfully"
    else
        echo "⚠️  Launch agent load reported warnings (this is often normal)"
    fi
    
    # Verify the service is actually running
    sleep 2
    if launchctl list | grep -q "com.zerolatency.doc-indexer"; then
        echo "✅ Zero-Latency daemon is running"
        echo "🌐 Service available at: http://localhost:8080"
        
        # Test if service responds
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            echo "🔍 Service health check passed"
        else
            echo "⚠️  Service may still be starting up..."
        fi
    else
        echo "❌ Failed to start daemon - check logs at /tmp/zero-latency-error.log"
    fi
}

# Function to uninstall launch agent
uninstall_daemon() {
    echo "🗑️  Uninstalling Zero-Latency daemon..."
    
    # Check if service is running
    if launchctl list | grep -q "com.zerolatency.doc-indexer"; then
        echo "🛑 Stopping running service..."
        launchctl unload "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    else
        echo "ℹ️  Service was not running"
    fi
    
    # Remove launch agent plist
    if [ -f "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST" ]; then
        rm -f "$USER_LAUNCH_AGENTS/$LAUNCH_AGENT_PLIST"
        echo "🗑️  Removed launch agent configuration"
    fi
    
    # Verify service is stopped
    if launchctl list | grep -q "com.zerolatency.doc-indexer"; then
        echo "⚠️  Service may still be running - try again in a moment"
    else
        echo "✅ Zero-Latency daemon completely removed"
    fi
}

# Function to show status
show_status() {
    echo "🔍 Zero-Latency Service Status"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if launchctl list | grep -q "com.zerolatency.doc-indexer"; then
        echo "✅ Zero-Latency daemon is running"
        
        # Get PID and status
        local status_line=$(launchctl list | grep "com.zerolatency.doc-indexer")
        local pid=$(echo "$status_line" | awk '{print $1}')
        echo "📊 Process ID: $pid"
        
        # Test service health
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            echo "🌐 Service available at: http://localhost:8080"
            echo "🔍 Health check: PASSED"
        else
            echo "⚠️  Service port not responding (may be starting up)"
        fi
        
        echo "🔧 CLI tool: $APP_DIR/mdx"
        echo "📋 Logs: /tmp/zero-latency.log"
        echo "❌ Errors: /tmp/zero-latency-error.log"
    else
        echo "❌ Zero-Latency daemon is not running"
        echo ""
        echo "To start the service, choose option 1"
    fi
}

# Function to open CLI
open_cli() {
    echo "🖥️  Opening Zero-Latency CLI..."
    osascript << EOD
tell application "Terminal"
    do script "cd '$APP_DIR' && echo 'Zero-Latency CLI Ready! Try: ./mdx search \"your query\"' && zsh || bash"
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
cat > "$LAUNCH_AGENTS_DIR/com.zerolatency.doc-indexer.plist" << 'PLISTEOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"\>
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.zerolatency.doc-indexer</string>
    <key>ProgramArguments</key>
    <array>
        <string>{{APP_DIR}}/doc-indexer</string>
        <string>--port</string>
        <string>8080</string>
        <string>--docs-path</string>
        <string>{{APP_DIR}}/../Resources/docs</string>
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
PLISTEOF

echo "✅ App bundle created: $BUNDLE_DIR"
echo "🚀 Double-click $BUNDLE_DIR to run!"
