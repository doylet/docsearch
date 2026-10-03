# Documentation Organization

**Last Updated**: December 2025

## Structure Overview

```
docs/
├── README.md                    # Main entry point (consolidated from README + INDEX)
├── CURRENT_ARCHITECTURE.md     # System architecture
├── CLI_REFERENCE.md            # CLI command reference
├── API_REFERENCE.md            # REST API specification
│
├── guides/                     # Comprehensive guides
│   ├── README.md
│   ├── DOCKER.md
│   ├── MONOREPO.md
│   ├── PROJECT_OVERVIEW.md
│   └── RELEASE_NOTES.md
│
├── architecture/               # Architecture implementation
├── adr/                        # Architecture Decision Records
├── implementation/             # Implementation guides
├── milestones/                 # Project milestones
├── sprint/                     # Sprint documentation
├── strategy/                   # Development strategy
│
├── issues/                     # Known issues & troubleshooting
│   └── SEARCH_FILTERING_TROUBLESHOOTING.md
│
├── services/                   # Service-specific docs
├── evaluation/                 # Evaluation reports
│
├── archive/                    # Archived historical docs
│   └── MILESTONE_HISTORY_COMPREHENSIVE.md
│
└── misc/                       # Miscellaneous artifacts
    └── artefacts/
```

## Key Changes

### ✅ Consolidated Entry Points
- **Before**: `README.md` and `INDEX.md` (duplicate content)
- **After**: Single `README.md` with clear navigation

### ✅ Organized Guides
- **Before**: Guides scattered in root (`DOCKER.md`, `MONOREPO.md`, etc.)
- **After**: All guides in `guides/` subdirectory

### ✅ Moved Troubleshooting
- **Before**: `SEARCH_FILTERING_TROUBLESHOOTING.md` in root
- **After**: Moved to `issues/` directory

### ✅ Archived Historical Docs
- **Before**: `MILESTONE_HISTORY_COMPREHENSIVE.md` in root
- **After**: Moved to `archive/` directory

### ✅ Removed Temporary Files
- Removed `ROOT_CLEANUP_SUMMARY.md`
- Removed duplicate `INDEX.md`

## Root-Level Files (4 files)

Only essential reference documentation remains at root:

1. **README.md** - Main documentation entry point
2. **CURRENT_ARCHITECTURE.md** - System architecture
3. **CLI_REFERENCE.md** - CLI command reference
4. **API_REFERENCE.md** - REST API specification

## Navigation

### For Quick Reference
- Start with [README.md](README.md)
- Check [CURRENT_ARCHITECTURE.md](CURRENT_ARCHITECTURE.md) for system overview
- Use [CLI_REFERENCE.md](CLI_REFERENCE.md) or [API_REFERENCE.md](API_REFERENCE.md) for commands

### For Detailed Guides
- See [guides/](guides/) for comprehensive guides
- Check [issues/](issues/) for troubleshooting
- Review [adr/](adr/) for design decisions

### For Historical Context
- Browse [milestones/](milestones/) for project history
- Check [archive/](archive/) for archived documentation
- See [sprint/](sprint/) for sprint documentation

---

**Result**: Clean, organized, and easy to navigate documentation structure.
