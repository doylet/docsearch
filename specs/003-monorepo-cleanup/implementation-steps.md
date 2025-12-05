# Implementation Steps: Monorepo Cleanup

**Feature**: 003-monorepo-cleanup
**Total Time**: ~2 hours
**Status**: Ready to execute

## Quick Reference

```bash
# Full migration script
./specs/003-monorepo-cleanup/migrate.sh
```

## Step-by-Step Guide

### Step 1: Pre-Migration Verification (10 min)

**Objective**: Ensure clean starting state

```bash
# Stop all containers
docker-compose down

# Check git status
git status

# Compare directories to see differences
diff -r frontend/ apps/frontend/ --brief | grep -v ".next\|node_modules" > /tmp/frontend-diff.txt
cat /tmp/frontend-diff.txt
```

**Success Criteria**:
- ✅ All containers stopped (`docker ps` shows nothing)
- ✅ Git status clean or changes committed
- ✅ Difference report generated

---

### Step 2: Consolidate Source to apps/frontend/ (15 min)

**Objective**: Copy latest files, resolve symlinks

```bash
# Copy source directories with latest versions (overwrite)
rsync -av --delete frontend/app/ apps/frontend/app/
rsync -av --delete frontend/application/ apps/frontend/application/
rsync -av --delete frontend/domain/ apps/frontend/domain/
rsync -av --delete frontend/infrastructure/ apps/frontend/infrastructure/
rsync -av --delete frontend/presentation/ apps/frontend/presentation/
rsync -av frontend/public/ apps/frontend/public/

# Resolve all symlinks to real files
cd apps/frontend/
for link in $(find . -type l -maxdepth 1); do
  if [ -e "$link" ]; then
    target=$(readlink -f "$link")
    rm "$link"
    cp "$target" "$link"
  fi
done
cd ../..

# Verify no symlinks remain
find apps/frontend/ -type l
# Should output nothing
```

**Success Criteria**:
- ✅ All source copied to apps/frontend/
- ✅ Latest changes preserved (indexing/page.tsx, globals.css)
- ✅ Zero symlinks remaining
- ✅ All configs are real files

---

### Step 3: Update Docker Configuration (15 min)

**Objective**: Point Docker to apps/frontend/

**Edit docker-compose.yml**:
```yaml
services:
  frontend:
    build:
      context: .                              # Root context
      dockerfile: apps/frontend/Dockerfile    # Dockerfile location
    volumes:
      - ./apps/frontend:/app                  # Source mount
      - /app/node_modules
      - /app/.next
```

**Edit apps/frontend/Dockerfile**:
```dockerfile
# Adjust COPY commands to expect build context at repo root
FROM node:20-alpine AS deps
WORKDIR /app
COPY apps/frontend/package*.json ./
RUN npm ci

FROM node:20-alpine AS builder
WORKDIR /app
COPY apps/frontend/ .
COPY --from=deps /app/node_modules ./node_modules
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
EXPOSE 3000
CMD ["node", "server.js"]
```

**Success Criteria**:
- ✅ docker-compose.yml references apps/frontend/
- ✅ Dockerfile COPY commands use apps/frontend/ paths
- ✅ Volume mounts point to apps/frontend/

---

### Step 4: Update Documentation & Scripts (20 min)

**Objective**: Fix all path references

```bash
# Update README
sed -i '' 's|cd frontend|cd apps/frontend|g' README.md
sed -i '' 's|./frontend/|./apps/frontend/|g' README.md

# Update Makefile
sed -i '' 's|cd frontend|cd apps/frontend|g' Makefile
sed -i '' 's|./frontend|./apps/frontend|g' Makefile

# Update MONOREPO.md if exists
if [ -f MONOREPO.md ]; then
  sed -i '' 's|/frontend/|/apps/frontend/|g' MONOREPO.md
fi

# Search for remaining hardcoded paths
echo "Searching for hardcoded paths..."
grep -r "cd frontend" . --exclude-dir={node_modules,.next,target,.git} || echo "None found"
grep -r "./frontend/" . --exclude-dir={node_modules,.next,target,.git} || echo "None found"
```

**Success Criteria**:
- ✅ README references apps/frontend/
- ✅ Makefile targets use apps/frontend/
- ✅ No hardcoded /frontend/ paths remain

---

### Step 5: Delete /frontend/ Directory (5 min)

**Objective**: Remove duplicate

```bash
# Create backup (insurance)
tar -czf /tmp/frontend-backup-$(date +%Y%m%d-%H%M%S).tar.gz frontend/
echo "Backup created at: /tmp/frontend-backup-$(date +%Y%m%d-%H%M%S).tar.gz"

# Delete frontend directory
git rm -rf frontend/

# Verify deletion
ls -la | grep frontend
# Should show nothing (or only apps/frontend via apps/ listing)
```

**Success Criteria**:
- ✅ /frontend/ deleted
- ✅ Backup exists in /tmp/
- ✅ Only apps/frontend/ remains

---

### Step 6: Verification & Testing (30 min)

**Objective**: Verify everything works

```bash
# Clean build from scratch
docker-compose down -v
docker-compose build --no-cache frontend
docker-compose up -d

# Wait for healthy
echo "Waiting for services..."
sleep 10
docker-compose ps

# Test search API
echo "Testing search..."
curl -X POST http://localhost:8081/api/search \
  -H "Content-Type: application/json" \
  -d '{"query":"test","collection":"zero_latency_docs"}' | jq

# Test indexing API
echo "Testing index..."
curl -X POST http://localhost:8081/api/index \
  -H "Content-Type: application/json" \
  -d '{"path":"/app/demo-content","collection_name":"test"}' | jq
```

**Manual Browser Tests**:
1. Open http://localhost:3000/ (search page)
2. Verify search interface loads
3. Verify input text is readable (dark text)
4. Click "Index Documents" in nav
5. Verify http://localhost:3000/indexing loads
6. Verify indexing form is styled correctly
7. Test search query
8. Test navigation between pages

**Success Criteria**:
- ✅ Docker build succeeds without errors
- ✅ Both services healthy
- ✅ Search API returns results
- ✅ Index API works
- ✅ Frontend pages load correctly
- ✅ Navigation works
- ✅ Input styling correct
- ✅ Build time ≤ previous

---

### Step 7: Commit & Document (15 min)

**Objective**: Commit with clear documentation

```bash
# Review changes
git status
git diff docker-compose.yml
git diff apps/frontend/Dockerfile

# Stage all changes
git add -A

# Commit
git commit -m "Consolidate frontend to apps/frontend/ (003-monorepo-cleanup)

Problem:
- Duplicate source in /frontend/ and /apps/frontend/
- Edits to /frontend/ didn't appear in Docker builds
- Hybrid symlink/copy structure caused constant confusion

Solution:
- Moved all source to apps/frontend/ (single source of truth)
- Resolved all symlinks to real files
- Updated Docker configs to use apps/frontend/
- Updated docker-compose.yml volume mounts and build context
- Updated README, Makefile, and docs
- Deleted /frontend/ directory entirely

Verification:
✅ Clean build succeeds without errors
✅ All features work (search, indexing, navigation)
✅ Hot reload works (<30s)
✅ Zero duplicate files
✅ Build time unchanged

Closes P1 blocker - developers can now edit files with immediate build reflection

FR-001 through FR-012: VERIFIED
SC-001 through SC-006: MET"

# Show commit
git log --oneline -1
git show --stat
```

**Success Criteria**:
- ✅ All changes committed
- ✅ Clear commit message
- ✅ Ready for PR and review

---

## Rollback Plan

If something goes wrong:

```bash
# Restore from backup
tar -xzf /tmp/frontend-backup-*.tar.gz

# Revert git changes
git reset --hard HEAD~1

# Rebuild
docker-compose down -v
docker-compose build
docker-compose up -d
```

---

## Success Metrics Summary

After completion, verify:

- ✅ **SC-001**: Edit file → save → rebuild → see changes in <30s
- ✅ **SC-002**: `find . -name "frontend" -type d` shows only `apps/frontend`
- ✅ **SC-003**: `docker-compose build` succeeds on first try
- ✅ **SC-004**: Search, indexing, navigation all work
- ✅ **SC-005**: New dev can follow README without encountering /frontend/
- ✅ **SC-006**: Build time same or better than before

---

## Quick Migration Script

```bash
#!/bin/bash
# migrate.sh - Full monorepo cleanup migration

set -e

echo "=== Step 1: Pre-flight checks ==="
docker-compose down
git status

echo "=== Step 2: Consolidate source ==="
rsync -av --delete frontend/app/ apps/frontend/app/
rsync -av --delete frontend/application/ apps/frontend/application/
rsync -av --delete frontend/domain/ apps/frontend/domain/
rsync -av --delete frontend/infrastructure/ apps/frontend/infrastructure/
rsync -av --delete frontend/presentation/ apps/frontend/presentation/

cd apps/frontend/
for link in $(find . -type l -maxdepth 1); do
  [ -e "$link" ] && rm "$link" && cp "$(readlink -f ../../frontend/$link)" "$link"
done
cd ../..

echo "=== Step 3: Update Docker configs ==="
# Manual step - edit docker-compose.yml and Dockerfile

echo "=== Step 4: Update docs ==="
sed -i '' 's|cd frontend|cd apps/frontend|g' README.md Makefile

echo "=== Step 5: Backup and delete ==="
tar -czf /tmp/frontend-backup-$(date +%Y%m%d).tar.gz frontend/
git rm -rf frontend/

echo "=== Step 6: Verify ==="
docker-compose build --no-cache frontend
docker-compose up -d

echo "=== Migration complete! ==="
echo "Test at: http://localhost:3000"
echo "Backup at: /tmp/frontend-backup-*.tar.gz"
```
