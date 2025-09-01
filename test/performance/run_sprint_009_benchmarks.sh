#!/bin/bash

# Sprint 009: Production Readiness Performance Validation
# ZL-009-002: Comprehensive performance testing and optimization

set -e

echo "🚀 Sprint 009 Production Performance Validation"
echo "=============================================="
echo ""

# Configuration
DOC_INDEXER_PID=""
RESULTS_DIR="./performance_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="${RESULTS_DIR}/sprint_009_performance_report_${TIMESTAMP}.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    if [ ! -z "$DOC_INDEXER_PID" ]; then
        echo "Stopping doc-indexer (PID: $DOC_INDEXER_PID)"
        kill $DOC_INDEXER_PID 2>/dev/null || true
        wait $DOC_INDEXER_PID 2>/dev/null || true
    fi
}

# Set trap for cleanup
trap cleanup EXIT

# Start doc-indexer service
start_service() {
    echo -e "${BLUE}🚀 Starting doc-indexer service...${NC}"
    cd ../../services/doc-indexer
    cargo run --bin doc-indexer -- --port 8081 &
    DOC_INDEXER_PID=$!
    cd ../../test/performance
    
    echo "Doc-indexer started with PID: $DOC_INDEXER_PID"
    echo "Waiting for service to be ready..."
    
    # Wait for service to be ready
    for i in {1..30}; do
        if curl -s http://localhost:8081/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Service is ready!${NC}"
            return 0
        fi
        echo "Waiting... ($i/30)"
        sleep 2
    done
    
    echo -e "${RED}❌ Service failed to start or become ready${NC}"
    exit 1
}

# Create results directory
setup_results() {
    mkdir -p "$RESULTS_DIR"
    echo "# Sprint 009 Performance Validation Report" > "$REPORT_FILE"
    echo "**Generated:** $(date)" >> "$REPORT_FILE"
    echo "**Test Suite:** ZL-009-002 Production Performance Validation" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
}

# Run performance tests
run_benchmarks() {
    echo -e "\n${BLUE}📊 Running comprehensive performance benchmarks...${NC}"
    
    echo "## Performance Test Results" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Test categories
    local tests=(
        "test_sustained_load_performance"
        "test_memory_usage_optimization" 
        "test_cache_optimization"
        "test_database_connection_optimization"
        "test_throughput_validation"
    )
    
    local test_names=(
        "Sustained Load Performance"
        "Memory Usage Optimization"
        "Cache Optimization"
        "Database Connection Optimization" 
        "Throughput Validation"
    )
    
    for i in "${!tests[@]}"; do
        local test="${tests[$i]}"
        local name="${test_names[$i]}"
        
        echo -e "\n${YELLOW}🧪 Running: $name${NC}"
        echo "### $name" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        
        if cargo test "$test" --release -- --nocapture >> "$REPORT_FILE" 2>&1; then
            echo -e "${GREEN}✅ $name: PASSED${NC}"
            echo "**Status:** ✅ PASSED" >> "$REPORT_FILE"
        else
            echo -e "${RED}❌ $name: FAILED${NC}"
            echo "**Status:** ❌ FAILED" >> "$REPORT_FILE"
        fi
        
        echo "" >> "$REPORT_FILE"
    done
}

# Performance validation summary
generate_summary() {
    echo -e "\n${BLUE}📋 Generating performance summary...${NC}"
    
    echo "## Performance Validation Summary" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Target metrics from ZL-009-002
    cat >> "$REPORT_FILE" << 'EOF'
### Target Metrics (ZL-009-002)

| Metric | Target | Status |
|--------|--------|--------|
| P95 Search Latency | <350ms | To be validated |
| P95 Reranking Latency | <900ms | To be validated |
| Sustained Throughput | >100 QPS | To be validated |
| Cache Hit Rate | >80% | To be validated |
| Memory Growth | <100MB | To be validated |
| Success Rate | >99% | To be validated |

### Performance Optimization Areas

1. **Load Testing**: Validated realistic query patterns with concurrent load
2. **Memory Management**: Monitored allocation patterns and garbage collection
3. **Cache Efficiency**: Measured cache hit rates and lookup performance
4. **Database Pooling**: Tested connection pooling under concurrent load
5. **Throughput Scaling**: Validated sustained QPS targets

### Production Readiness Assessment

- [ ] All performance targets met
- [ ] Memory usage within acceptable bounds
- [ ] Cache optimization effective (>80% hit rate)
- [ ] Database connections properly pooled
- [ ] Sustained throughput achieved (>100 QPS)
- [ ] System stability under load confirmed

EOF
}

# Main execution
main() {
    echo -e "${BLUE}Starting Sprint 009 Production Performance Validation...${NC}"
    
    setup_results
    start_service
    run_benchmarks
    generate_summary
    
    echo -e "\n${GREEN}🎉 Performance validation complete!${NC}"
    echo -e "📄 Report generated: ${BLUE}$REPORT_FILE${NC}"
    echo ""
    echo "To view the report:"
    echo "  cat $REPORT_FILE"
    echo ""
    echo "Next steps for ZL-009-002:"
    echo "  1. Review performance metrics against targets"
    echo "  2. Identify optimization opportunities"
    echo "  3. Implement memory and cache improvements"
    echo "  4. Validate database connection pooling"
    echo "  5. Document performance baselines"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "sprint_009_benchmarks.rs" ]; then
    echo -e "${RED}❌ Please run this script from the test/performance directory${NC}"
    exit 1
fi

# Run main function
main "$@"
