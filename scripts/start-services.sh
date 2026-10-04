#!/bin/bash
set -e

echo "🚀 Starting DocSearch Services..."
echo ""

# Check if backend binary exists
if [ ! -f "target/release/doc-indexer" ]; then
    echo "📦 Building backend (first time only, this may take 5-10 minutes)..."
    cargo build --release --bin doc-indexer
fi

# Start backend in background
echo "🔧 Starting backend on port 8081..."
./target/release/doc-indexer --port 8081 &
BACKEND_PID=$!

# Wait for backend to be ready
echo "⏳ Waiting for backend to start..."
sleep 3

# Start frontend
echo "🎨 Starting frontend on port 3000..."
cd frontend && npm run dev &
FRONTEND_PID=$!

echo ""
echo "✅ Services started!"
echo ""
echo "   Frontend: http://localhost:3000"
echo "   Backend:  http://localhost:8081"
echo ""
echo "Press Ctrl+C to stop both services"
echo ""

# Wait for either process to exit
wait $BACKEND_PID $FRONTEND_PID
