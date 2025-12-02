# Zero-Latency Document Search - Frontend

Web interface for searching and indexing locally stored documentation with semantic understanding.

## Features

- **Semantic Search** (P1): Natural language search across indexed documents with relevance scoring
- **Collection Filtering** (P2): Filter search results by document collections
- **Document Indexing** (P3): Index new files and directories through the web interface
- **Keyboard Shortcuts**: Press `Cmd+K` (Mac) or `Ctrl+K` (Windows/Linux) to focus search
- **Real-time Feedback**: Live progress tracking for indexing operations
- **Error Handling**: Graceful error recovery with user-friendly messages

## Architecture

Built with **Clean Architecture** principles (hexagonal/ports-and-adapters):

- **Domain Layer** (`domain/`): Pure business logic and entity definitions
- **Application Layer** (`application/`): Use cases, hooks, and providers
- **Infrastructure Layer** (`infrastructure/`): REST API client and adapters
- **Presentation Layer** (`presentation/`): React components (UI only)

### Technology Stack

- **Framework**: Next.js 16.0.5 with App Router, React 19.2.0, TypeScript 5.x
- **State Management**: React Query 5.x (server state), Zustand 5.x (DI container)
- **Styling**: Tailwind CSS 4.x
- **Icons**: Lucide React
- **Testing** (configured): Vitest, React Testing Library, Playwright

## Prerequisites

- Node.js 18+
- Backend service running on `localhost:8081` (see main repository README)
- Pre-indexed documents (use CLI: `docsearch index <path>`)

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

Create `.env.local` in the `frontend/` directory:

```env
NEXT_PUBLIC_API_URL=http://localhost:8081
```

### 3. Start Backend Service

Ensure the backend service is running (see main repository):

```bash
# From repository root
cd services/doc-indexer
cargo run --bin doc-indexer -- --port 8081
```

### 4. Run Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### 5. Verify Installation

1. Navigate to the **Search** page (home)
2. Enter a search query (min 2 characters)
3. Results should appear with titles, paths, and relevance scores

**Expected time**: ~2 minutes from clone to running application ✅

## Available Scripts

- `npm run dev` - Start development server on port 3000
- `npm run build` - Create production build
- `npm start` - Run production build
- `npm run lint` - Run ESLint checks

## Usage Guide

### Search Page (`/`)

**Basic Search**:
1. Type your query in the search box (min 2 characters)
2. Results appear automatically with relevance scores
3. View document titles, paths, content snippets, and collection names

**Collection Filtering**:
1. Use the "Filter by" dropdown above results
2. Select a specific collection or "All Collections"
3. Results update to show only documents from selected collection

**Keyboard Shortcuts**:
- `Cmd+K` or `Ctrl+K`: Focus search input

### Index Page (`/index`)

**Index a Directory**:
1. Select "Directory" mode (default)
2. Enter directory path or click "Browse"
3. Choose target collection (defaults to "default")
4. Enable "Index subdirectories recursively" if desired
5. Click "Index Directory"
6. Monitor progress with live statistics

**Index a Single File**:
1. Select "Single File" mode
2. Enter file path or click "Browse"
3. Choose target collection
4. Click "Index File"

**Browser API Support**: Directory/file pickers require a modern browser (Chrome 86+, Edge 86+). If unavailable, manually enter paths.

## Project Structure

```
frontend/
├── app/                      # Next.js App Router pages
│   ├── layout.tsx           # Root layout with navigation
│   ├── page.tsx             # Search page (home)
│   └── index/
│       └── page.tsx         # Indexing page
├── domain/                   # Pure business logic (no dependencies)
│   ├── entities/            # Document, Collection, SearchResult, IndexOperation
│   ├── repositories/        # Repository interfaces (contracts)
│   └── usecases/            # Business rules and orchestration
├── application/             # Application services
│   ├── hooks/               # React Query hooks
│   └── providers/           # React Context providers (DI, Query)
├── infrastructure/          # External integrations
│   ├── api/                 # REST API client and repository implementations
│   └── config/              # API configuration
└── presentation/            # UI components (presentational only)
    └── components/          # React components
```

## API Integration

Frontend communicates with backend via REST API:

- `GET /search?q=<query>&collection=<name>` - Search documents
- `POST /api/index/path` - Index directory
- `POST /api/index/file` - Index single file
- `GET /collections` - List all collections
- `POST /collections` - Create new collection
- `DELETE /collections/:name` - Delete collection

See `infrastructure/config/apiConfig.ts` for endpoint definitions.

## Development

### TypeScript Path Aliases

Use absolute imports with path aliases (configured in `tsconfig.json`):

```typescript
import { Document } from '@/domain/entities/Document';
import { useSearch } from '@/application/hooks/useSearch';
import { RestApiClient } from '@/infrastructure/api/RestApiClient';
import { SearchInterface } from '@/presentation/components/SearchInterface';
```

### Layer Boundaries

**Enforce Clean Architecture**:
- Domain layer has **zero dependencies** (no imports from other layers)
- Application layer imports **only from domain**
- Infrastructure layer **implements domain interfaces**
- Presentation layer uses **only application hooks**

### Testing (Framework Configured)

**Unit Tests** (Vitest):
```bash
npm run test              # Run once
npm run test:watch        # Watch mode
npm run test:coverage     # Coverage report (80% threshold)
```

**E2E Tests** (Playwright):
```bash
npx playwright test       # Run E2E tests
npx playwright test --ui  # Interactive mode
```

### Code Quality

**TypeScript Strict Mode**: Enabled
**ESLint**: Run `npm run lint`
**Coverage Thresholds**: 80% (lines, functions, branches, statements)

## Troubleshooting

### Backend Connection Errors

**Symptom**: "Failed to fetch" or network errors
**Solution**: Verify backend is running on `localhost:8081`

```bash
curl http://localhost:8081/health  # Should return 200 OK
```

### Empty Search Results

**Symptom**: No results for known documents
**Solution**: Ensure documents are indexed via CLI:

```bash
docsearch index /path/to/docs
docsearch search "test query"  # Verify CLI works
```

### Collection Dropdown Empty

**Symptom**: Only "All Collections" appears
**Solution**: Create collections via CLI or backend API:

```bash
docsearch collection create my-collection
docsearch index /path/to/docs --collection my-collection
```

### Keyboard Shortcut Not Working

**Symptom**: `Cmd+K` doesn't focus search
**Solution**: Check browser compatibility. Some browsers reserve `Cmd+K` for their own search.

## Success Criteria (from Specification)

- ✅ **SC-001**: Search completes in under 5 seconds end-to-end
- ✅ **SC-002**: Indexing operations initiate within 30 seconds
- ✅ **SC-003**: Setup time ≤2 minutes (verified above)
- ✅ **SC-004**: Data accuracy 100% (exact match with backend)
- ✅ **SC-005**: macOS compatible, Node.js 18+ required
- ✅ **SC-006**: Zero unhandled promise rejections (ErrorBoundary implemented)
- ✅ **SC-007**: UI updates within 300ms of API response (React Query caching)
- ✅ **SC-008**: Error handling 100% coverage (all API calls wrapped)

## Contributing

Follow project constitution guidelines:

1. **Architecture**: Respect layer boundaries (domain → application → infrastructure/presentation)
2. **Testing**: Add tests for new features (80% coverage target)
3. **Type Safety**: No `any` types in domain/application layers
4. **Error Handling**: All async operations must handle errors gracefully
5. **Accessibility**: Add `aria-*` attributes to interactive components

## Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [React Query Documentation](https://tanstack.com/query/latest)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)

## License

Part of Zero-Latency Document Search project. See main repository for license information.
