# Collection Domain Schemas

This directory contains domain-specific schema definitions that are referenced by the main OpenAPI specification.

## Domain Organization

- `collection.yaml` - Collection management schemas
- `document.yaml` - Document and content schemas  
- `search.yaml` - Search operation schemas
- `analytics.yaml` - Analytics and monitoring schemas
- `common.yaml` - Shared/common schemas

## Usage

These schemas are included in the main OpenAPI specification using `$ref` references and provide modular, maintainable schema definitions organized by business domain.

## Tenant Patterns

All domain schemas follow multi-tenant patterns with:
- `TenantResource` base schema for resources
- Optional `tenant_id` fields for multi-tenant deployments  
- Tenant-aware filtering and scoping
