# Monorepo Guide for DocSearch

## Overview

DocSearch uses a monorepo structure powered by **Turborepo** for efficient builds and intelligent caching. This guide covers the structure, workflows, and best practices.

## Repository Structure

```
docsearch/
├── apps/                    # Applications
│   ├── backend/            # Rust backend service
│   │   ├── package.json   # Turbo wrapper for cargo commands
│   │   └── (symlinks to ../../services/doc-indexer)
│   └── frontend/           # Next.js frontend
│       └── (symlinks to ../../frontend)
├── packages/               # Shared packages
│   └── rust-crates/       # Existing crates/ moved here
│       └── [zero-latency-* crates]
├── crates/                # Legacy location (symlinked)
├── services/              # Legacy location (symlinked)
├── frontend/              # Legacy location (symlinked)
├── package.json           # Root workspace configuration
├── turbo.json             # Turborepo pipeline configuration
└── Cargo.toml             # Rust workspace manifest
```

## Quick Start

### Install Dependencies

```bash
# Install npm dependencies (including turbo)
npm install

# Cargo dependencies installed automatically on build
```

### Build Everything

```bash
# Build all apps and packages
npm run build
# Or use turbo directly
turbo build
```

### Development Mode

```bash
# Run all apps in development mode with hot reload
npm run dev

# Run specific app
turbo dev --filter=@docsearch/frontend
turbo dev --filter=@docsearch/backend
```

### Run Tests

```bash
# Test all packages
npm run test

# Test specific package
turbo test --filter=@docsearch/backend
```

## Turborepo Benefits

### 1. Intelligent Caching

Turborepo caches build outputs and only rebuilds what changed:

```bash
# First build (cold)
turbo build  # Takes full time

# Second build (warm cache)
turbo build  # Completes in <5 seconds
```

**Cache is stored in:** `.turbo/cache/`

### 2. Parallel Execution

Tasks run in parallel when there are no dependencies:

```bash
# Lints frontend and backend simultaneously
npm run lint
```

### 3. Incremental Builds

Only affected packages rebuild:

```bash
# Change frontend code
turbo build  # Only rebuilds frontend, skips backend

# Change shared crate
turbo build  # Rebuilds backend + anything that depends on the crate
```

## Pipeline Configuration

The `turbo.json` defines how tasks relate:

```json
{
  "pipeline": {
    "build": {
      "dependsOn": ["^build"],  # Build dependencies first
      "outputs": ["dist/**", ".next/**", "target/release/**"]
    },
    "dev": {
      "cache": false,           # Don't cache dev servers
      "persistent": true        # Keep running
    },
    "test": {
      "dependsOn": ["^build"],  # Test after building dependencies
      "outputs": ["coverage/**"]
    }
  }
}
```

## Working with the Monorepo

### Adding a New Package

1. Create directory under `packages/`:
   ```bash
   mkdir packages/my-package
   ```

2. Add `package.json`:
   ```json
   {
     "name": "@docsearch/my-package",
     "version": "1.0.0",
     "private": true
   }
   ```

3. Add to root `package.json` workspaces (automatic with `packages/*`)

### Adding a New App

1. Create directory under `apps/`:
   ```bash
   mkdir apps/my-app
   ```

2. Add `package.json` with Turbo scripts:
   ```json
   {
     "name": "@docsearch/my-app",
     "scripts": {
       "build": "...",
       "dev": "...",
       "test": "..."
     }
   }
   ```

### Rust Integration

Backend uses a wrapper `package.json` that calls cargo commands:

```json
{
  "scripts": {
    "build": "cargo build --release --bin doc-indexer",
    "dev": "cargo watch -x 'run --bin doc-indexer'",
    "test": "cargo test --workspace",
    "lint": "cargo clippy --workspace"
  }
}
```

This allows Turborepo to:
- Cache cargo build artifacts
- Track dependencies between Rust crates and Node packages
- Run cargo commands in parallel with Node builds

## Common Tasks

### Clean Everything

```bash
# Clean all build artifacts
npm run clean

# Or manually
rm -rf .turbo node_modules apps/*/node_modules apps/*/.next
cargo clean
```

### View Dependency Graph

```bash
# See what depends on what
turbo run build --dry --graph
```

### Force Rebuild (Ignore Cache)

```bash
# Rebuild everything from scratch
turbo build --force
```

### Run Task in Specific Package

```bash
# Filter by package name
turbo build --filter=@docsearch/frontend

# Filter by directory
turbo build --filter=./apps/backend
```

## Build Performance

### Baseline (Without Turborepo)

- Cold build: ~4 minutes
- Incremental: ~2-3 minutes (manual dependency tracking)

### With Turborepo

- Cold build: ~4 minutes (first time)
- Warm cache: <5 seconds (nothing changed)
- Incremental: <1 minute (only affected packages)

**Target**: 50% reduction in build time ✅

## Docker Integration

### Production Builds

The Dockerfile works with monorepo structure:

```bash
# Build from root (includes all workspaces)
docker-compose build
```

### Development with Hot Reload

```bash
# Mount source directories for hot reload
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up
```

## Troubleshooting

### "Package not found" Errors

**Problem**: Workspace not recognized

**Solution**:
```bash
# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install
```

### Stale Cache Issues

**Problem**: Changes not reflecting in build

**Solution**:
```bash
# Clear Turbo cache
rm -rf .turbo
turbo build --force
```

### Cargo and npm Out of Sync

**Problem**: Cargo builds succeed but Turbo fails

**Solution**:
```bash
# Rebuild cargo first
cargo clean && cargo build --release

# Then run Turbo
turbo build --filter=@docsearch/backend --force
```

### Symlink Issues

**Problem**: Symlinks broken after migration

**Solution**:
```bash
# Recreate symlinks
cd apps/frontend && ln -sf ../../frontend/* .
cd apps/backend && ln -sf ../../services/doc-indexer/* .
```

## Migration Status

### Current State

- ✅ Root package.json with workspaces
- ✅ turbo.json with pipeline configuration
- ✅ apps/backend/ wrapper created
- ✅ apps/frontend/ structure created
- ⏳ Symbolic links for backward compatibility
- ⏳ Docker compose updates pending
- ⏳ Full migration to apps/ structure

### Backward Compatibility

During migration, the old structure is maintained via symbolic links:
- `frontend/` → `apps/frontend/`
- `services/doc-indexer/` → `apps/backend/`
- `crates/` → `packages/rust-crates/`

This allows existing tools (Make, Docker, IDEs) to continue working during the transition.

## Next Steps

1. **Complete Migration**: Move all files to apps/ and packages/
2. **Update Docker**: Point builds to new structure
3. **Update Documentation**: Reflect new paths in all docs
4. **Team Training**: Ensure everyone understands Turborepo workflows
5. **CI/CD Integration**: Update pipeline to use Turbo caching

## Resources

- [Turborepo Documentation](https://turbo.build/repo/docs)
- [Monorepo Best Practices](https://turbo.build/repo/docs/handbook)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
