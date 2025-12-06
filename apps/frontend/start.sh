#!/bin/bash

# DocSearch Quick Start Script

echo "🚀 Starting DocSearch..."
echo ""

# Check if doc-indexer is running
if ! curl -s http://localhost:8081/health > /dev/null 2>&1; then
    echo "⚠️  Doc-indexer service not running on port 8081"
    echo "   Start it with: cargo run --bin doc-indexer -- --port 8081"
    echo ""
    read -p "Start doc-indexer now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cd .. && cargo run --bin doc-indexer -- --port 8081 &
        DOC_INDEXER_PID=$!
        echo "   Started doc-indexer (PID: $DOC_INDEXER_PID)"
        echo "   Waiting for service to be ready..."
        sleep 3
    fi
else
    echo "✅ Doc-indexer service is running"
fi

echo ""
echo "🌐 Starting frontend on http://localhost:3000"
echo ""

npm run dev
