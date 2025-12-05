'use client';

import { SearchResult } from '@/domain/entities/SearchResult';
import { FileText } from 'lucide-react';

interface SearchResultsProps {
  results: SearchResult[];
}

/**
 * SearchResults Component
 *
 * Presentation component for displaying search results with metadata.
 * Part of User Story 1 (P1) - Semantic Document Search.
 */
export function SearchResults({ results }: SearchResultsProps) {
  return (
    <div className="space-y-4">
      {results.map((result) => (
        <div
          key={result.document.id}
          className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow"
        >
          <div className="flex items-start gap-3">
            <FileText className="w-5 h-5 text-blue-500 mt-1 flex-shrink-0" />
            <div className="flex-1 min-w-0">
              <h3 className="text-lg font-semibold text-gray-900 mb-1 truncate">
                {result.document.title || result.document.path}
              </h3>

              <p className="text-sm text-gray-600 mb-2 truncate" title={result.document.path}>
                {result.document.path}
              </p>

              {result.document.content && (
                <p className="text-gray-700 mb-3 line-clamp-3">
                  {result.document.content.substring(0, 300)}
                  {result.document.content.length > 300 ? '...' : ''}
                </p>
              )}

              {result.highlights && result.highlights.length > 0 && (
                <div className="mb-3 p-2 bg-yellow-50 border-l-4 border-yellow-400 rounded">
                  <p className="text-sm text-gray-700 italic">
                    {result.highlights[0]}
                  </p>
                </div>
              )}

              <div className="flex items-center gap-4 text-sm text-gray-500">
                <span className="flex items-center gap-1">
                  <span className="font-medium">Score:</span>
                  <span className="text-blue-600 font-semibold">
                    {(result.score * 100).toFixed(1)}%
                  </span>
                </span>

                {result.rank && (
                  <span className="flex items-center gap-1">
                    <span className="font-medium">Rank:</span>
                    <span>#{result.rank}</span>
                  </span>
                )}

                <span className="flex items-center gap-1">
                  <span className="font-medium">Collection:</span>
                  <span className="px-2 py-0.5 bg-gray-100 rounded text-xs">
                    {result.document.collection}
                  </span>
                </span>
              </div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
