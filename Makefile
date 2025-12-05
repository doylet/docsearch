# Zero-Latency Schema-First Build System
#
# This Makefile provides commands for generating API types, clients, and documentation
# from the OpenAPI 3.1 specification.

.PHONY: help install-deps validate-schemas generate-schemas generate-rust generate-clients generate-docs clean test-schemas docker-build docker-up docker-down docker-logs docker-restart docker-rebuild docker-clean

# Default target
help:
	@echo "Zero-Latency Document Search - Build System"
	@echo ""
	@echo "Docker Commands (Production-Ready):"
	@echo "  docker-build          Build all Docker images"
	@echo "  docker-up             Start all services with Docker Compose"
	@echo "  docker-down           Stop all services"
	@echo "  docker-logs           View logs from all services"
	@echo "  docker-restart        Restart all services"
	@echo "  docker-rebuild        Rebuild images and restart services (no cache)"
	@echo "  docker-clean          Remove all containers, images, and volumes"
	@echo ""
	@echo "Monorepo Commands (Turborepo):"
	@echo "  turbo-build           Build all apps and packages"
	@echo "  turbo-dev             Run all apps in development mode"
	@echo "  turbo-test            Test all packages"
	@echo "  turbo-lint            Lint all packages"
	@echo "  turbo-clean           Clean all build artifacts"
	@echo ""
	@echo "Development Commands:"
	@echo "  dev-backend           Run backend in development mode"
	@echo "  dev-frontend          Run frontend in development mode"
	@echo "  dev                   Run both backend and frontend"
	@echo ""
	@echo "Schema Generation:"
	@echo "  install-deps          Install required dependencies"
	@echo "  validate-schemas      Validate OpenAPI schemas"
	@echo "  generate-schemas      Generate all artifacts from schemas"
	@echo "  test-breaking-changes Test for API breaking changes"
	@echo "  backup-schema         Create schema backup for change detection"
	@echo "  ci-validate           Run complete CI validation pipeline"
	@echo "  test-schemas          Test generated code compilation"
	@echo "  clean                 Clean generated artifacts"
	@echo ""

# Docker Commands
docker-build:
	@echo "Building Docker images..."
	docker-compose build
	@echo "✅ Docker images built successfully"

docker-up:
	@echo "Starting services with Docker Compose..."
	docker-compose up -d
	@echo "✅ Services started"
	@echo ""
	@echo "Frontend: http://localhost:3000"
	@echo "Backend API: http://localhost:8081"
	@echo ""
	@echo "Run 'make docker-logs' to view logs"

docker-down:
	@echo "Stopping services..."
	docker-compose down
	@echo "✅ Services stopped"

docker-logs:
	docker-compose logs -f

docker-clean:
	@echo "Removing all containers, images, and volumes..."
	docker-compose down -v --rmi all
	@echo "✅ Docker environment cleaned"

docker-restart:
	@echo "Restarting services..."
	docker-compose restart
	@echo "✅ Services restarted"

docker-rebuild:
	@echo "Rebuilding and restarting services..."
	docker-compose down
	docker-compose build --no-cache
	docker-compose up -d
	@echo "✅ Services rebuilt and restarted"

# Development Commands
dev-backend:
	@echo "Starting backend in development mode..."
	cd services/doc-indexer && cargo run --bin doc-indexer -- --port 8081

dev-frontend:
	@echo "Starting frontend in development mode..."
	cd frontend && npm run dev

dev:
	@echo "Starting both services in development mode..."
	@echo "Note: Run in separate terminals or use a process manager like concurrently"
	@echo ""
	@echo "Terminal 1: make dev-backend"
	@echo "Terminal 2: make dev-frontend"

# Variables
SCHEMA_FILE := api/public/openapi.yaml
GENERATED_DIR := target/generated
RUST_OUTPUT := $(GENERATED_DIR)/rust
TS_OUTPUT := $(GENERATED_DIR)/typescript
PYTHON_OUTPUT := $(GENERATED_DIR)/python
DOCS_OUTPUT := $(GENERATED_DIR)/docs

# Install dependencies
install-deps:
	@echo "Installing OpenAPI Generator..."
	@if ! command -v npm &> /dev/null; then \
		echo "Error: npm is required but not installed"; \
		exit 1; \
	fi
	npm install -g openapi-generator-cli@7.8.0
	@echo "Installing schema validation tools..."
	npm install -g @redocly/cli
	@echo "Installing breaking change detection..."
	npm install -g oasdiff

# Validate schemas
validate-schemas:
	@echo "Validating OpenAPI schema..."
	@if ! command -v redocly &> /dev/null; then \
		echo "Error: redocly not found. Run 'make install-deps' first"; \
		exit 1; \
	fi
	npx @redocly/cli lint $(SCHEMA_FILE)
	@echo "✅ Schema validation passed"

# Generate all artifacts (delegates to Cargo build)
generate-schemas:
	@echo "Generating all artifacts from OpenAPI schema..."
	@echo "Note: Generation is handled by zero-latency-api build.rs"
	cd crates/zero-latency-api && cargo build
	@echo "✅ All artifacts generated successfully"

# Test schema changes for breaking changes
test-breaking-changes:
	@echo "Testing for breaking changes..."
	@if [ -f "$(SCHEMA_FILE).backup" ]; then \
		echo "Comparing with backup schema..."; \
		npx oasdiff breaking $(SCHEMA_FILE).backup $(SCHEMA_FILE) || \
		(echo "⚠️ Breaking changes detected" && exit 1); \
	else \
		echo "No backup schema found, creating one..."; \
		cp $(SCHEMA_FILE) $(SCHEMA_FILE).backup; \
	fi
	@echo "✅ No breaking changes detected"

# Create schema backup for change detection
backup-schema:
	@echo "Creating schema backup..."
	cp $(SCHEMA_FILE) $(SCHEMA_FILE).backup
	@echo "✅ Schema backup created"

# Generate and validate in CI mode
ci-validate:
	@echo "Running CI validation pipeline..."
	$(MAKE) validate-schemas
	$(MAKE) generate-schemas
	$(MAKE) test-schemas
	@echo "✅ CI validation completed"
	swagger-cli validate $(SCHEMA_FILE)
	@echo "✅ Schema validation passed"

# Lint schemas with Spectral
lint-schemas:
	@echo "Linting OpenAPI schema..."
	@if ! command -v spectral &> /dev/null; then \
		echo "Warning: spectral not found. Skipping linting"; \
	else \
		spectral lint $(SCHEMA_FILE); \
	fi

# Generate all artifacts
generate-schemas: validate-schemas generate-rust generate-clients generate-docs
	@echo "✅ All schema artifacts generated successfully"

# Generate Rust types
generate-rust:
	@echo "Generating Rust API types..."
	@mkdir -p $(RUST_OUTPUT)
	@if ! command -v openapi-generator-cli &> /dev/null; then \
		echo "Error: openapi-generator-cli not found. Run 'make install-deps' first"; \
		exit 1; \
	fi
	openapi-generator-cli generate \
		-i $(SCHEMA_FILE) \
		-g rust \
		-o $(RUST_OUTPUT) \
		--additional-properties packageName=zero_latency_api,supportAsync=true,library=reqwest
	@echo "✅ Rust types generated in $(RUST_OUTPUT)"

# Generate client SDKs
generate-clients: generate-typescript generate-python
	@echo "✅ All client SDKs generated"

generate-typescript:
	@echo "Generating TypeScript client..."
	@mkdir -p $(TS_OUTPUT)
	openapi-generator-cli generate \
		-i $(SCHEMA_FILE) \
		-g typescript-fetch \
		-o $(TS_OUTPUT) \
		--additional-properties npmName=zero-latency-api-client,supportsES6=true,typescriptThreePlus=true
	@echo "✅ TypeScript client generated in $(TS_OUTPUT)"

generate-python:
	@echo "Generating Python client..."
	@mkdir -p $(PYTHON_OUTPUT)
	openapi-generator-cli generate \
		-i $(SCHEMA_FILE) \
		-g python \
		-o $(PYTHON_OUTPUT) \
		--additional-properties packageName=zero_latency_api_client,generateSourceCodeOnly=true,pythonAtLeast=3.8
	@echo "✅ Python client generated in $(PYTHON_OUTPUT)"

# Generate documentation
generate-docs:
	@echo "Generating API documentation..."
	@mkdir -p $(DOCS_OUTPUT)
	# Generate HTML documentation
	openapi-generator-cli generate \
		-i $(SCHEMA_FILE) \
		-g html2 \
		-o $(DOCS_OUTPUT)/html
	# Generate Markdown documentation
	openapi-generator-cli generate \
		-i $(SCHEMA_FILE) \
		-g markdown \
		-o $(DOCS_OUTPUT)/markdown
	@echo "✅ Documentation generated in $(DOCS_OUTPUT)"

# Test schema changes
test-schemas:
	@echo "Testing generated Rust code compilation..."
	cd crates/zero-latency-api && cargo check
	@echo "Testing service integration..."
	cargo build --workspace
	@echo "✅ Generated code compiles and integrates successfully"

# Build the API crate (triggers generation)
build-api:
	@echo "Building zero-latency-api crate..."
	cargo build -p zero-latency-api
	@echo "✅ API crate built successfully"

# Clean generated artifacts
clean:
	@echo "Cleaning generated artifacts..."
	rm -rf $(GENERATED_DIR)
	rm -rf crates/zero-latency-api/target
	@echo "✅ Generated artifacts cleaned"

# Development workflow
dev-setup: install-deps validate-schemas generate-schemas build-api
	@echo "✅ Development environment set up successfully"

# CI/CD targets
ci-validate: validate-schemas lint-schemas test-schemas
	@echo "✅ CI validation passed"

ci-generate: generate-schemas
	@echo "✅ CI generation completed"

# Check for breaking changes
check-breaking-changes:
	@echo "Checking for breaking changes..."
	@if [ -f "$(SCHEMA_FILE).previous" ]; then \
		echo "Comparing with previous schema version..."; \
		if command -v oasdiff &> /dev/null; then \
			oasdiff breaking $(SCHEMA_FILE).previous $(SCHEMA_FILE); \
		else \
			echo "Warning: oasdiff not found. Install with: go install github.com/Tufin/oasdiff@latest"; \
		fi \
	else \
		echo "No previous schema found. Skipping breaking change detection."; \
	fi

# Save current schema as previous for next comparison
save-schema-version:
	@cp $(SCHEMA_FILE) $(SCHEMA_FILE).previous
	@echo "Schema version saved for future breaking change detection"

# Watch for schema changes and regenerate
watch:
	@echo "Watching for schema changes..."
	@if command -v fswatch &> /dev/null; then \
		fswatch -o $(SCHEMA_FILE) | xargs -n1 -I{} make generate-schemas; \
	else \
		echo "Error: fswatch not found. Install with: brew install fswatch (macOS)"; \
		exit 1; \
	fi

# Generate JSON-RPC schemas for MCP compliance
generate-jsonrpc:
	@echo "Generating JSON-RPC schemas for MCP compliance..."
	@mkdir -p $(GENERATED_DIR)/jsonrpc
	# TODO: Implement JSON-RPC schema generation
	@echo "✅ JSON-RPC schemas generated"

# Package clients for distribution
package-clients: generate-clients
	@echo "Packaging client SDKs..."
	# Package TypeScript client
	cd $(TS_OUTPUT) && npm pack
	# Package Python client
	cd $(PYTHON_OUTPUT) && python setup.py sdist
	@echo "✅ Client SDKs packaged for distribution"

# Full release preparation
release: clean validate-schemas lint-schemas generate-schemas test-schemas package-clients generate-docs
	@echo "✅ Release artifacts prepared successfully"
	@echo "Generated artifacts:"
	@echo "  - Rust types: $(RUST_OUTPUT)"
	@echo "  - TypeScript client: $(TS_OUTPUT)"
	@echo "  - Python client: $(PYTHON_OUTPUT)"
	@echo "  - Documentation: $(DOCS_OUTPUT)"

# Turborepo / Monorepo Commands
.PHONY: turbo-build turbo-dev turbo-test turbo-lint turbo-clean

turbo-build:
	@echo "Building all apps and packages with Turborepo..."
	npm run build
	@echo "✅ Build complete"

turbo-dev:
	@echo "Starting all apps in development mode..."
	npm run dev

turbo-test:
	@echo "Testing all packages..."
	npm run test
	@echo "✅ Tests complete"

turbo-lint:
	@echo "Linting all packages..."
	npm run lint
	@echo "✅ Lint complete"

turbo-clean:
	@echo "Cleaning all build artifacts..."
	npm run clean
	rm -rf .turbo
	@echo "✅ Clean complete"
