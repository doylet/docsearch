'use client';

import { forwardRef } from 'react';
import { Search } from 'lucide-react';

interface SearchInterfaceProps {
  query: string;
  onQueryChange: (query: string) => void;
  isLoading?: boolean;
  error?: Error | null;
}

/**
 * SearchInterface Component
 *
 * Presentation component for search input with validation feedback.
 * Part of User Story 1 (P1) - Semantic Document Search.
 * Supports keyboard shortcuts (Cmd+K or Ctrl+K) via ref forwarding.
 */
export const SearchInterface = forwardRef<HTMLInputElement, SearchInterfaceProps>(function SearchInterface({
  query,
  onQueryChange,
  isLoading = false,
  error
}, ref) {
  const isQueryTooShort = query.length > 0 && query.length < 2;

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
        <input
          ref={ref}
          type="text"
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          placeholder="Search documentation... (min 2 characters)"
          className={`w-full pl-10 pr-4 py-3 border rounded-lg focus:outline-none focus:ring-2 transition-colors text-gray-900 placeholder:text-gray-400 ${
            isQueryTooShort
              ? 'border-red-300 focus:ring-red-500'
              : 'border-gray-300 focus:ring-blue-500'
          }`}
          aria-label="Search query"
          aria-invalid={isQueryTooShort}
          aria-describedby={isQueryTooShort ? 'query-error' : undefined}
        />
        {isLoading && (
          <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
            <div className="animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full" />
          </div>
        )}
      </div>

      {isQueryTooShort && (
        <p id="query-error" className="mt-2 text-sm text-red-600">
          Please enter at least 2 characters to search
        </p>
      )}

      {error && (
        <div className="mt-3 p-3 bg-red-50 border border-red-200 rounded-md">
          <p className="text-sm text-red-800">
            <strong>Error:</strong> {error.message || 'Failed to search. Please try again.'}
          </p>
        </div>
      )}
    </div>
  );
});
