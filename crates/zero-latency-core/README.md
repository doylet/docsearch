# zero-latency-core

Core domain models, types, and abstractions for the Zero-Latency documentation search system.

## Overview

This crate provides the foundational building blocks used across all Zero-Latency components:

- **Domain Models**: Document, metadata, and search result types
- **Error Handling**: Comprehensive error types and result handling
- **Health Monitoring**: System health check abstractions
- **Common Traits**: Shared interfaces for services and repositories
- **Configuration**: Type-safe configuration management
- **Utilities**: Common helper functions and macros

## Key Types

### Domain Models
- `Document`: Core document representation with metadata
- `SearchResult`: Search result with relevance scoring
- `DocumentMetadata`: File system and content metadata
- `HealthStatus`: System health monitoring types

### Error Handling
- `Result<T>`: Standardized result type for all operations
- `Error`: Comprehensive error enumeration
- Error conversion traits for seamless error handling

### Configuration
- Configuration traits and validation
- Environment variable management
- Type-safe configuration builders

## Usage

```rust
use zero_latency_core::{Document, Result, Error};

// Core document handling
let document = Document::new("content", "path/to/file.md")?;

// Standardized error handling
fn process_document(doc: Document) -> Result<SearchResult> {
    // Implementation
}
```

## Dependencies

This is a foundational crate with minimal dependencies:
- `serde` for serialization
- `uuid` for unique identifiers
- `chrono` for timestamp handling
- `thiserror` for error handling

## Feature Flags

- `default`: All standard features
- `serde`: Serialization support (enabled by default)

## Architecture

This crate follows clean architecture principles:
- Domain-driven design with clear boundaries
- Dependency inversion through traits
- Immutable data structures where possible
- Comprehensive error handling
