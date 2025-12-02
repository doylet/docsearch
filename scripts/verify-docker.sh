#!/bin/bash
set -e

echo "🔍 Verifying Docker setup..."
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker Desktop."
    exit 1
fi
echo "✅ Docker installed"

# Check Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose."
    exit 1
fi
echo "✅ Docker Compose installed"

# Check files
echo ""
echo "Checking required files..."
[ -f "docker-compose.yml" ] && echo "✅ docker-compose.yml" || echo "❌ docker-compose.yml missing"
[ -f "frontend/Dockerfile" ] && echo "✅ frontend/Dockerfile" || echo "❌ frontend/Dockerfile missing"
[ -f "Dockerfile" ] && echo "✅ Dockerfile (backend)" || echo "❌ Dockerfile missing"
[ -f "Makefile" ] && echo "✅ Makefile" || echo "❌ Makefile missing"

echo ""
echo "🎉 Docker setup verified!"
echo ""
echo "Next steps:"
echo "  1. make docker-build    # Build images"
echo "  2. make docker-up       # Start services"
echo "  3. Open http://localhost:3000"
