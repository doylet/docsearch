# Zero-Latency Documentation

**Version**: v0.1.0
**Last Updated**: December 2025
**Status**: Production Ready

---

## 🎯 Start Here

### Essential Documentation

1. **[CURRENT_ARCHITECTURE.md](CURRENT_ARCHITECTURE.md)** - System overview and architecture
2. **[CLI_REFERENCE.md](CLI_REFERENCE.md)** - Complete CLI command reference
3. **[API_REFERENCE.md](API_REFERENCE.md)** - REST API specification
4. **[../README.md](../README.md)** - Installation and quick start guide

### Guides

- **[Docker Guide](guides/DOCKER.md)** - Docker setup and deployment
- **[Monorepo Guide](guides/MONOREPO.md)** - Turborepo structure and workflows
- **[Project Overview](guides/PROJECT_OVERVIEW.md)** - Comprehensive project overview
- **[Release Notes](guides/RELEASE_NOTES.md)** - Version history

---

## 📚 Documentation Structure

### Reference Documentation
- **[CLI_REFERENCE.md](CLI_REFERENCE.md)** - Command-line interface
- **[API_REFERENCE.md](API_REFERENCE.md)** - REST API endpoints
- **[CURRENT_ARCHITECTURE.md](CURRENT_ARCHITECTURE.md)** - System architecture

### Architecture & Design
- **[architecture/](architecture/)** - Architecture implementation details
- **[adr/](adr/)** - Architecture Decision Records
- **[implementation/](implementation/)** - Implementation guides

### Development
- **[sprint/](sprint/)** - Sprint plans and summaries
- **[strategy/](strategy/)** - Development strategy and roadmaps
- **[milestones/](milestones/)** - Project milestones

### Operations
- **[issues/](issues/)** - Known issues and troubleshooting
- **[services/](services/)** - Service-specific documentation

### Historical
- **[archive/](archive/)** - Archived historical documentation
- **[misc/](misc/)** - Miscellaneous artifacts

---

## 🚀 Quick Start

```bash
# Build the project
cargo build --release

# Start the server
mdx server

# Create a collection
mdx collection create my-docs

# Index documents
mdx index /path/to/documents --collection my-docs

# Search documents
mdx search "your query" --collection my-docs
```

---

## 📖 Navigation Guide

### For New Users
1. Read [CURRENT_ARCHITECTURE.md](CURRENT_ARCHITECTURE.md) for system overview
2. Review [CLI_REFERENCE.md](CLI_REFERENCE.md) for command-line usage
3. Check [API_REFERENCE.md](API_REFERENCE.md) for programmatic access
4. See [../README.md](../README.md) for installation

### For Developers
1. Study [CURRENT_ARCHITECTURE.md](CURRENT_ARCHITECTURE.md) for system design
2. Review [architecture/](architecture/) for implementation patterns
3. Check [adr/](adr/) for design decisions
4. See [implementation/](implementation/) for technical details

### For Operations
1. Use [CLI_REFERENCE.md](CLI_REFERENCE.md) for operational commands
2. Monitor via [API_REFERENCE.md](API_REFERENCE.md) health endpoints
3. Check [issues/](issues/) for known issues
4. Review [guides/DOCKER.md](guides/DOCKER.md) for deployment

---

## 🎯 Current Capabilities

### Collection Management
- Create, read, update, delete collections
- Collection statistics and health monitoring

### Document Operations
- Index documents from filesystem
- Read-only document discovery
- Semantic search with relevance scoring

### API & CLI
- Complete REST API
- Comprehensive CLI interface
- Multiple output formats (JSON, table, YAML)

---

## 🔗 Related Documentation

- **Main Project**: [../README.md](../README.md)
- **Changelog**: [../CHANGELOG.md](../CHANGELOG.md)
- **Agent System**: [../agents/README.md](../agents/README.md)

---

**Last Updated**: December 2025
