'use client';

import { Search, FileQuestion } from 'lucide-react';

interface EmptyStateProps {
  type: 'no-query' | 'no-results';
  query?: string;
}

/**
 * EmptyState Component
 *
 * Presentation component for empty states in search.
 * Part of User Story 1 (P1) - Semantic Document Search.
 */
export function EmptyState({ type, query }: EmptyStateProps) {
  if (type === 'no-query') {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <Search className="w-16 h-16 text-gray-300 mb-4" />
        <h3 className="text-xl font-semibold text-gray-700 mb-2">
          Start Searching
        </h3>
        <p className="text-gray-500 max-w-md">
          Enter a search query above to find documents in your indexed collections.
          Semantic search understands natural language queries.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <FileQuestion className="w-16 h-16 text-gray-300 mb-4" />
      <h3 className="text-xl font-semibold text-gray-700 mb-2">
        No Results Found
      </h3>
      <p className="text-gray-500 max-w-md">
        No documents matched your search for <strong>&quot;{query}&quot;</strong>.
        Try different keywords or check if documents are indexed.
      </p>
    </div>
  );
}
