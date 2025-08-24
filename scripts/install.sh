#!/bin/bash
# Simple installer script for Zero-Latency

set -e

INSTALL_DIR="/usr/local/bin"
SERVICE_NAME="com.zerolatency.doc-indexer"
PLIST_PATH="$HOME/Library/LaunchAgents/$SERVICE_NAME.plist"

echo "🚀 Zero-Latency Installer"
echo "========================="

# Function to install binaries
install_binaries() {
    echo "📦 Installing binaries to $INSTALL_DIR..."
    
    # Check if we have permission
    if [[ ! -w "$INSTALL_DIR" ]]; then
        echo "🔐 Need sudo access to install to $INSTALL_DIR"
        sudo cp target/release/doc-indexer "$INSTALL_DIR/"
        sudo cp target/release/mdx "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/doc-indexer"
        sudo chmod +x "$INSTALL_DIR/mdx"
    else
        cp target/release/doc-indexer "$INSTALL_DIR/"
        cp target/release/mdx "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/doc-indexer"
        chmod +x "$INSTALL_DIR/mdx"
    fi

    ln -s "$INSTALL_DIR/mdx" "$HOME/bin/mdx"

    echo "✅ Binaries installed"
}

# Function to create and load daemon
install_daemon() {
    echo "🔧 Setting up background daemon..."
    
    # Create launch agent directory
    mkdir -p "$(dirname "$PLIST_PATH")"
    
    # Create launch agent plist
    cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$SERVICE_NAME</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/doc-indexer</string>
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
</dict>
</plist>
EOF
    
    # Load the daemon
    launchctl load "$PLIST_PATH"
    
    echo "✅ Daemon installed and started"
}

# Function to check status
check_status() {
    if launchctl list | grep -q "$SERVICE_NAME"; then
        echo "✅ Zero-Latency daemon is running"
        echo "🌐 Service: http://localhost:8080"
        echo "🔧 CLI: mdx --help"
    else
        echo "❌ Daemon is not running"
    fi
}

# Function to uninstall
uninstall() {
    echo "🗑️  Uninstalling Zero-Latency..."
    
    # Stop and remove daemon
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    rm -f "$PLIST_PATH"
    
    # Remove binaries
    if [[ -w "$INSTALL_DIR" ]]; then
        rm -f "$INSTALL_DIR/doc-indexer"
        rm -f "$INSTALL_DIR/mdx"
    else
        sudo rm -f "$INSTALL_DIR/doc-indexer"
        sudo rm -f "$INSTALL_DIR/mdx"
    fi
    
    echo "✅ Uninstalled successfully"
}

# Main menu
case "${1:-menu}" in
    install)
        install_binaries
        install_daemon
        check_status
        echo ""
        echo "🎉 Installation complete!"
        echo "💡 Try: mdx search 'your query'"
        ;;
    uninstall)
        uninstall
        ;;
    status)
        check_status
        ;;
    menu|*)
        echo "Usage: $0 {install|uninstall|status}"
        echo ""
        echo "Commands:"
        echo "  install   - Install binaries and start daemon"
        echo "  uninstall - Remove binaries and stop daemon"
        echo "  status    - Check if daemon is running"
        ;;
esac
