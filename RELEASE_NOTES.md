# Zero-Latency v1.0.0 Release Notes

**Release Date**: August 22, 2025  
**Distribution**: Zero-Latency-v1.0.0.dmg (6.4MB)

## 🎉 Major Release: Production-Ready Distribution

This release marks the first production-ready version of Zero-Latency Documentation Search, featuring a complete macOS distribution package with professional installer and self-contained binaries.

## 🚀 New Features

### Professional macOS Distribution

- **DMG Installer**: Professional macOS installer package (`Zero-Latency-v1.0.0.dmg`)
- **App Bundle**: Native `Zero-Latency.app` with integrated GUI control panel
- **LaunchAgent Integration**: Automatic background daemon management
- **Drag-and-Drop Installation**: Standard macOS user experience

### Self-Contained Binaries

- **Embedded ONNX Runtime**: No external library dependencies using `download-binaries` feature
- **Single Binary Deployment**: Complete ML inference stack in each binary (~8.4MB)
- **Zero Configuration**: No manual library path setup required

### Enhanced User Experience

- **GUI Control Panel**: Easy daemon start/stop via app interface
- **Integrated CLI Terminal**: Access command-line tools from app
- **Professional Documentation**: In-app README and usage instructions
- **Multiple Installation Methods**: App bundle, command-line, or from source

## 🔧 Technical Improvements

### Build System

- **Automated Packaging**: Complete build and distribution scripts
  - `scripts/build-macos-app.sh`: Creates app bundle with LaunchAgent
  - `scripts/build-dmg.sh`: Packages into distributable DMG
  - `scripts/install.sh`: Command-line installation option

- **Simplified Dependencies**: Cleaned up Cargo.toml configuration
  - Removed duplicate `ort` dependency
  - Unified ONNX Runtime configuration
  - Eliminated manual dylib path management

### Architecture Enhancements

- **Service Layer Improvements**: Enhanced error handling and configuration
- **Production Logging**: Structured logging with multiple output formats
- **Health Monitoring**: Comprehensive service health checks

## 📦 Distribution Packages

### macOS App Bundle (Recommended)

```bash
# Download Zero-Latency-v1.0.0.dmg (6.4MB)
# Double-click to mount
# Drag Zero-Latency.app to Applications
# Launch app for GUI control panel
```

**Features**:
- ✅ Self-contained binaries with embedded ONNX Runtime
- ✅ Automatic daemon management via macOS LaunchAgent  
- ✅ GUI control panel for easy management
- ✅ Integrated CLI terminal access
- ✅ Professional macOS installer experience

### Command Line Installation

```bash
# Quick install script
./scripts/install.sh

# Manual installation
sudo cp target/release/{doc-indexer,mdx} /usr/local/bin/
```

### From Source

```bash
git clone <repo-url>
cd Zero-Latency
cargo build --release
# Binaries available at ./target/release/{doc-indexer,mdx}
```

## 📊 Performance Metrics

- **Binary Size**: ~8.4MB per binary (self-contained)
- **DMG Size**: 6.4MB (complete distribution)
- **Model Cache**: ~/.cache/zero-latency/models/ (~126MB)
- **Search Latency**: 10-20ms typical response time
- **Memory Usage**: ~100MB base + model size

## 🛠 Installation Guide

### Quick Start (macOS)

1. **Download**: Get `Zero-Latency-v1.0.0.dmg` from releases
2. **Install**: Drag `Zero-Latency.app` to Applications folder
3. **Launch**: Double-click app to open control panel
4. **Start**: Click "Install & Start Daemon" to begin
5. **Use**: Access CLI via app terminal or system command line

### CLI Usage

```bash
# Search documents (after daemon is running)
mdx search "embedding model"

# Check system status
mdx status

# Get help
mdx help
```

### API Access

```bash
# REST API (default port 8080)
curl -X POST "http://localhost:8080/api/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "vector search", "limit": 5}'
```

## 🔄 Upgrade Path

### From Previous Versions

This is the first official release. Previous development versions can be replaced by:

1. Stop any running services: `mdx server --stop`
2. Install new version using DMG or build scripts
3. Restart daemon via app control panel

### Configuration Compatibility

- **Settings**: Previous configuration files remain compatible
- **Model Cache**: Existing model cache is preserved
- **Data**: Document indices are maintained

## 🐛 Known Issues

- **First Launch**: Initial model download may take 30-60 seconds
- **Qdrant Dependency**: Vector database must be running (Docker recommended)
- **macOS Only**: Current distribution package targets macOS only

## 🔮 Coming Soon

- **Windows Distribution**: Native Windows installer and app
- **Linux Packages**: Debian/RPM packages for Linux distributions
- **Web Interface**: Browser-based search UI
- **Enhanced CLI**: Additional management commands

## 📚 Documentation

- **README**: Updated with comprehensive installation and usage guide
- **Changelog**: Complete history of all changes and improvements
- **API Documentation**: Full REST API reference and examples
- **Build Guide**: Instructions for creating custom distributions

## 🙏 Acknowledgments

Built with:
- **Rust**: High-performance system programming
- **ONNX Runtime**: Efficient ML model inference
- **Qdrant**: Vector similarity search
- **gte-small**: Local embedding model

---

**Download**: [Zero-Latency-v1.0.0.dmg](releases/Zero-Latency-v1.0.0.dmg)  
**Documentation**: [README.md](README.md)  
**Source Code**: [GitHub Repository](https://github.com/your-repo/zero-latency)

For support and questions, please open an issue on GitHub.
