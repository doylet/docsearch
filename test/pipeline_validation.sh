#!/bin/bash
set -e

# Enhanced Search Pipeline Validation Script
# Tests all feature combinations and measures performance

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/test/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}🚀 Zero-Latency Enhanced Search Pipeline Validation${NC}"
echo -e "${BLUE}=================================================${NC}"
echo "Timestamp: $TIMESTAMP"
echo "Project Root: $PROJECT_ROOT"
echo "Results Directory: $RESULTS_DIR"
echo ""

# Test configuration
TEST_DOCS_DIR="$PROJECT_ROOT/docs"
TEST_PORT_BASE=8090
PERFORMANCE_ITERATIONS=10

# Feature combinations to test
declare -a FEATURES=("embedded" "cloud" "full")
declare -a TEST_QUERIES=("architecture" "vector database" "JSON-RPC" "embedding model" "search performance")

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

measure_time() {
    local start_time=$(date +%s%N)
    eval "$1"
    local end_time=$(date +%s%N)
    local duration=$((($end_time - $start_time) / 1000000))
    echo $duration
}

cleanup_servers() {
    log_info "Cleaning up any running servers..."
    pkill -f "doc-indexer" || true
    sleep 2
}

build_feature_variant() {
    local feature=$1
    local build_dir="$PROJECT_ROOT/target/test-$feature"
    
    log_info "Building $feature variant..."
    
    cd "$PROJECT_ROOT/services/doc-indexer"
    
    if [ "$feature" = "embedded" ]; then
        cargo build --release --features embedded --no-default-features --target-dir "$build_dir"
    elif [ "$feature" = "cloud" ]; then
        cargo build --release --features cloud --no-default-features --target-dir "$build_dir"
    elif [ "$feature" = "full" ]; then
        cargo build --release --features full --target-dir "$build_dir"
    else
        log_error "Unknown feature: $feature"
        return 1
    fi
    
    if [ $? -eq 0 ]; then
        log_success "Built $feature variant successfully"
        echo "$build_dir/release/doc-indexer"
    else
        log_error "Failed to build $feature variant"
        return 1
    fi
}

test_server_startup() {
    local binary=$1
    local feature=$2
    local port=$3
    local timeout=10
    
    log_info "Testing server startup for $feature variant on port $port..."
    
    # Start server in background
    "$binary" --port "$port" > "$RESULTS_DIR/server_$feature.log" 2>&1 &
    local server_pid=$!
    
    # Wait for startup
    local count=0
    while [ $count -lt $timeout ]; do
        if curl -s "http://localhost:$port/health" > /dev/null 2>&1; then
            log_success "Server started successfully (PID: $server_pid)"
            echo $server_pid
            return 0
        fi
        sleep 1
        count=$((count + 1))
    done
    
    log_error "Server failed to start within $timeout seconds"
    kill $server_pid 2>/dev/null || true
    return 1
}

test_json_rpc_transport() {
    local port=$1
    local feature=$2
    
    log_info "Testing JSON-RPC transport for $feature variant..."
    
    # Test service info
    local info_response=$(curl -s -X POST "http://localhost:$port/jsonrpc" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"service.info","id":1}')
    
    if echo "$info_response" | jq -e '.result.name' > /dev/null 2>&1; then
        log_success "JSON-RPC service.info successful"
    else
        log_error "JSON-RPC service.info failed: $info_response"
        return 1
    fi
    
    # Test health check
    local health_response=$(curl -s -X POST "http://localhost:$port/jsonrpc" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"health.check","id":2}')
    
    if echo "$health_response" | jq -e '.result.status' > /dev/null 2>&1; then
        log_success "JSON-RPC health.check successful"
        return 0
    else
        log_error "JSON-RPC health.check failed: $health_response"
        return 1
    fi
}

index_test_documents() {
    local port=$1
    local feature=$2
    
    log_info "Indexing test documents for $feature variant..."
    
    # Index the docs directory
    local index_response=$(curl -s -X POST "http://localhost:$port/jsonrpc" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"document.index_path\",\"params\":{\"path\":\"$TEST_DOCS_DIR\",\"recursive\":true},\"id\":3}")
    
    if echo "$index_response" | jq -e '.result' > /dev/null 2>&1; then
        local indexed_count=$(echo "$index_response" | jq -r '.result.documents_indexed // 0')
        log_success "Indexed $indexed_count documents successfully"
        echo $indexed_count
        return 0
    else
        log_error "Document indexing failed: $index_response"
        return 1
    fi
}

benchmark_search_performance() {
    local port=$1
    local feature=$2
    local query=$3
    local iterations=$4
    
    log_info "Benchmarking search performance: '$query' ($iterations iterations)"
    
    local total_time=0
    local success_count=0
    local results_file="$RESULTS_DIR/search_benchmark_${feature}_${TIMESTAMP}.json"
    
    echo "[]" > "$results_file"
    
    for i in $(seq 1 $iterations); do
        local start_time=$(date +%s%N)
        
        local search_response=$(curl -s -X POST "http://localhost:$port/jsonrpc" \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"document.search\",\"params\":{\"query\":\"$query\",\"limit\":5},\"id\":$i}")
        
        local end_time=$(date +%s%N)
        local duration=$((($end_time - $start_time) / 1000000))
        
        if echo "$search_response" | jq -e '.result' > /dev/null 2>&1; then
            success_count=$((success_count + 1))
            total_time=$((total_time + duration))
            
            # Record detailed result
            local result_entry=$(echo "$search_response" | jq -c --arg iter "$i" --arg duration "$duration" --arg query "$query" '{
                iteration: ($iter | tonumber),
                query: $query,
                duration_ms: ($duration | tonumber),
                result_count: (.result.documents | length),
                timestamp: (now | strftime("%Y-%m-%d %H:%M:%S"))
            }')
            
            # Append to results file
            jq --argjson entry "$result_entry" '. += [$entry]' "$results_file" > "$results_file.tmp" && mv "$results_file.tmp" "$results_file"
            
            echo -n "."
        else
            echo -n "x"
        fi
    done
    
    echo ""
    
    if [ $success_count -gt 0 ]; then
        local avg_time=$((total_time / success_count))
        log_success "Search benchmark complete: $success_count/$iterations successful, avg ${avg_time}ms"
        echo $avg_time
    else
        log_error "All search requests failed"
        return 1
    fi
}

generate_performance_report() {
    local feature=$1
    local report_file="$RESULTS_DIR/performance_report_${feature}_${TIMESTAMP}.md"
    
    log_info "Generating performance report for $feature variant..."
    
    cat > "$report_file" << EOF
# Performance Report: $feature Feature Variant

**Generated**: $(date)
**Feature Set**: $feature
**Test Documents**: $TEST_DOCS_DIR

## Summary

EOF

    # Add benchmark results if available
    local benchmark_file="$RESULTS_DIR/search_benchmark_${feature}_${TIMESTAMP}.json"
    if [ -f "$benchmark_file" ]; then
        echo "## Search Performance Benchmarks" >> "$report_file"
        echo "" >> "$report_file"
        
        for query in "${TEST_QUERIES[@]}"; do
            local query_results=$(jq -r --arg q "$query" '.[] | select(.query == $q) | .duration_ms' "$benchmark_file")
            if [ -n "$query_results" ]; then
                local min_time=$(echo "$query_results" | sort -n | head -1)
                local max_time=$(echo "$query_results" | sort -n | tail -1)
                local avg_time=$(echo "$query_results" | awk '{sum+=$1} END {print sum/NR}')
                
                echo "### Query: \"$query\"" >> "$report_file"
                echo "- Min: ${min_time}ms" >> "$report_file"
                echo "- Max: ${max_time}ms" >> "$report_file"
                echo "- Avg: ${avg_time}ms" >> "$report_file"
                echo "" >> "$report_file"
            fi
        done
    fi
    
    log_success "Performance report generated: $report_file"
}

# Main test execution
main() {
    log_info "Starting enhanced search pipeline validation..."
    
    # Cleanup any existing servers
    cleanup_servers
    
    # Test each feature variant
    for feature in "${FEATURES[@]}"; do
        log_info "Testing feature variant: $feature"
        echo "----------------------------------------"
        
        # Skip cloud tests if external dependencies not available
        if [ "$feature" = "cloud" ]; then
            log_warning "Skipping cloud variant tests (requires external Qdrant/OpenAI setup)"
            continue
        fi
        
        # Build the variant
        local binary=$(build_feature_variant "$feature")
        if [ $? -ne 0 ]; then
            log_error "Failed to build $feature variant, skipping tests"
            continue
        fi
        
        # Test server startup
        local port=$((TEST_PORT_BASE + $(echo "$feature" | wc -c)))
        local server_pid=$(test_server_startup "$binary" "$feature" "$port")
        if [ $? -ne 0 ]; then
            log_error "Failed to start server for $feature variant, skipping tests"
            continue
        fi
        
        # Test JSON-RPC transport
        if test_json_rpc_transport "$port" "$feature"; then
            log_success "JSON-RPC transport validation passed for $feature"
        else
            log_error "JSON-RPC transport validation failed for $feature"
            kill $server_pid 2>/dev/null || true
            continue
        fi
        
        # Index test documents
        local doc_count=$(index_test_documents "$port" "$feature")
        if [ $? -eq 0 ]; then
            log_success "Document indexing completed: $doc_count documents"
        else
            log_error "Document indexing failed for $feature"
            kill $server_pid 2>/dev/null || true
            continue
        fi
        
        # Benchmark search performance
        for query in "${TEST_QUERIES[@]}"; do
            benchmark_search_performance "$port" "$feature" "$query" 5
        done
        
        # Generate performance report
        generate_performance_report "$feature"
        
        # Cleanup
        kill $server_pid 2>/dev/null || true
        sleep 2
        
        log_success "Completed testing for $feature variant"
        echo ""
    done
    
    log_success "Enhanced search pipeline validation complete!"
    log_info "Results available in: $RESULTS_DIR"
}

# Execute main function
main "$@"
