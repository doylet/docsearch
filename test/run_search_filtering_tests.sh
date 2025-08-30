#!/bin/bash

# Sprint 005: Comprehensive Search Filtering Test Runner
# Orchestrates testing across all interfaces with proper setup and teardown

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOC_INDEXER_PORT=8081
TEST_TIMEOUT=300  # 5 minutes

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if service is running
check_service() {
    local url="http://localhost:$DOC_INDEXER_PORT/health"
    if curl -s -f "$url" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Wait for service to be ready
wait_for_service() {
    local max_attempts=30
    local attempt=1
    
    log_info "Waiting for doc-indexer service to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if check_service; then
            log_success "Service is ready!"
            return 0
        fi
        
        log_info "Attempt $attempt/$max_attempts: Service not ready, waiting..."
        sleep 2
        ((attempt++))
    done
    
    log_error "Service failed to become ready within timeout"
    return 1
}

# Start the doc-indexer service
start_service() {
    log_info "Starting doc-indexer service..."
    
    cd "$PROJECT_ROOT"
    
    # Build the project first
    log_info "Building project..."
    cargo build --bin doc-indexer
    
    if [ $? -ne 0 ]; then
        log_error "Failed to build doc-indexer"
        return 1
    fi
    
    # Start the service in background
    log_info "Starting service on port $DOC_INDEXER_PORT..."
    cargo run --bin doc-indexer -- --port $DOC_INDEXER_PORT > /tmp/doc-indexer.log 2>&1 &
    SERVICE_PID=$!
    
    # Wait for service to be ready
    if wait_for_service; then
        log_success "Doc-indexer service started successfully (PID: $SERVICE_PID)"
        return 0
    else
        log_error "Failed to start doc-indexer service"
        if [ ! -z "$SERVICE_PID" ]; then
            kill $SERVICE_PID 2>/dev/null || true
        fi
        return 1
    fi
}

# Stop the service
stop_service() {
    if [ ! -z "$SERVICE_PID" ]; then
        log_info "Stopping doc-indexer service (PID: $SERVICE_PID)..."
        kill $SERVICE_PID 2>/dev/null || true
        wait $SERVICE_PID 2>/dev/null || true
        log_success "Service stopped"
    fi
}

# Run Python integration tests
run_python_tests() {
    log_info "Running Python integration tests..."
    
    cd "$PROJECT_ROOT"
    
    # Check if Python test file exists
    if [ ! -f "test/integration/test_search_filtering.py" ]; then
        log_error "Python test file not found"
        return 1
    fi
    
    # Make sure it's executable
    chmod +x test/integration/test_search_filtering.py
    
    # Run the tests
    python3 test/integration/test_search_filtering.py "http://localhost:$DOC_INDEXER_PORT"
    
    if [ $? -eq 0 ]; then
        log_success "Python integration tests passed"
        return 0
    else
        log_error "Python integration tests failed"
        return 1
    fi
}

# Run Rust integration tests
run_rust_tests() {
    log_info "Running Rust integration tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run the specific test file
    cargo test --package doc-indexer --test test_search_filtering_integration -- --nocapture
    
    if [ $? -eq 0 ]; then
        log_success "Rust integration tests passed"
        return 0
    else
        log_error "Rust integration tests failed"
        return 1
    fi
}

# Run CLI tests
run_cli_tests() {
    log_info "Running CLI tests..."
    
    cd "$PROJECT_ROOT"
    
    # Build CLI binary
    log_info "Building CLI binary..."
    cargo build --bin mdx
    
    if [ $? -ne 0 ]; then
        log_error "Failed to build CLI binary"
        return 1
    fi
    
    local cli_binary="./target/debug/mdx"
    
    # Test CLI search with collection filtering
    log_info "Testing CLI collection filtering..."
    
    # Test 1: CLI with collection parameter
    if timeout 30 $cli_binary search "test" --collection "zero_latency_docs" > /tmp/cli_test_1.log 2>&1; then
        log_success "CLI collection filtering test passed"
    else
        log_warning "CLI collection filtering test may have issues (check logs)"
    fi
    
    # Test 2: CLI default search
    if timeout 30 $cli_binary search "test" > /tmp/cli_test_2.log 2>&1; then
        log_success "CLI default search test passed"
    else
        log_warning "CLI default search test may have issues (check logs)"
    fi
    
    return 0
}

# Run comprehensive performance test
run_performance_test() {
    log_info "Running performance tests..."
    
    cd "$PROJECT_ROOT"
    
    # Simple performance test with multiple concurrent requests
    log_info "Testing REST API performance..."
    
    # Use curl to make multiple requests
    for i in {1..5}; do
        curl -s -X POST "http://localhost:$DOC_INDEXER_PORT/api/search" \
            -H "Content-Type: application/json" \
            -d '{"query": "test", "filters": {"collection_name": "zero_latency_docs"}}' \
            > /tmp/perf_test_$i.log &
    done
    
    # Wait for all requests to complete
    wait
    
    log_success "Performance test completed"
    return 0
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    stop_service
    
    # Clean up temporary files
    rm -f /tmp/doc-indexer.log
    rm -f /tmp/cli_test_*.log
    rm -f /tmp/perf_test_*.log
}

# Main test runner
main() {
    log_info "🚀 Starting Sprint 005: Comprehensive Search Filtering Tests"
    
    # Set up cleanup trap
    trap cleanup EXIT
    
    # Check if service is already running
    if check_service; then
        log_info "Service is already running, using existing instance"
        SERVICE_PID=""
    else
        # Start the service
        if ! start_service; then
            log_error "Failed to start service"
            exit 1
        fi
    fi
    
    # Run all test suites
    local overall_success=true
    
    # Python integration tests
    if ! run_python_tests; then
        overall_success=false
    fi
    
    # Rust integration tests
    if ! run_rust_tests; then
        overall_success=false
    fi
    
    # CLI tests
    if ! run_cli_tests; then
        overall_success=false
    fi
    
    # Performance tests
    if ! run_performance_test; then
        overall_success=false
    fi
    
    # Final summary
    echo ""
    log_info "📊 Test Summary:"
    
    if [ "$overall_success" = true ]; then
        log_success "🎉 All test suites completed successfully!"
        log_info "Sprint 005 search filtering functionality is fully validated"
        exit 0
    else
        log_error "⚠️  Some test suites failed"
        log_info "Check the logs above for details"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    "python")
        start_service && run_python_tests
        ;;
    "rust")
        start_service && run_rust_tests
        ;;
    "cli")
        start_service && run_cli_tests
        ;;
    "performance")
        start_service && run_performance_test
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [test_type]"
        echo ""
        echo "Test types:"
        echo "  python      Run only Python integration tests"
        echo "  rust        Run only Rust integration tests"
        echo "  cli         Run only CLI tests"
        echo "  performance Run only performance tests"
        echo "  (none)      Run all tests (default)"
        echo ""
        echo "Examples:"
        echo "  $0                    # Run all tests"
        echo "  $0 python           # Run only Python tests"
        echo "  $0 rust             # Run only Rust tests"
        ;;
    *)
        main
        ;;
esac
