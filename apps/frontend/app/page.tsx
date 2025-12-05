"use client";

import { useState, useRef } from "react";
import { useSearch } from "@/application/hooks/useSearch";
import { useCollections } from "@/application/hooks/useCollections";
import { useKeyboardShortcut } from "@/application/hooks/useKeyboardShortcut";
import { SearchInterface } from "@/presentation/components/SearchInterface";
import { SearchResults } from "@/presentation/components/SearchResults";
import { LoadingSpinner } from "@/presentation/components/LoadingSpinner";
import { EmptyState } from "@/presentation/components/EmptyState";
import { ErrorBoundary } from "@/presentation/components/ErrorBoundary";
import { CollectionSelector } from "@/presentation/components/CollectionSelector";

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const [collection, setCollection] = useState("all");
  const searchInputRef = useRef<HTMLInputElement>(null);

  const { data: collections, isLoading: isLoadingCollections, error: collectionsError } = useCollections();
  const { data: results, isLoading, error } = useSearch(query, {
    collection: collection === "all" ? undefined : collection
  });

  // Cmd+K or Ctrl+K to focus search
  useKeyboardShortcut('k', () => {
    searchInputRef.current?.focus();
  }, { metaKey: true });

  const showResults = results && results.length > 0;
  const showNoResults = results && results.length === 0 && query.length >= 2;
  const showEmptyState = !query || query.length < 2;

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-gray-50 py-8 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">
              Zero-Latency Document Search
            </h1>
            <p className="text-gray-600">
              Search your indexed documentation with semantic understanding
              <span className="ml-2 text-sm text-gray-500">
                (Press <kbd className="px-2 py-0.5 text-xs bg-gray-100 border border-gray-300 rounded">⌘K</kbd> to focus search)
              </span>
            </p>
          </div>

          <SearchInterface
            ref={searchInputRef}
            query={query}
            onQueryChange={setQuery}
            isLoading={isLoading}
            error={error || null}
          />

          <div className="mt-6">
            <div className="flex justify-between items-center mb-4">
              <CollectionSelector
                collections={collectionsError ? [] : collections}
                selectedCollection={collection}
                onCollectionChange={setCollection}
                isLoading={isLoadingCollections}
              />

              {showResults && (
                <p className="text-sm text-gray-600">
                  Found {results.length} result{results.length !== 1 ? 's' : ''}
                </p>
              )}
            </div>

            {collectionsError && (
              <div className="mb-4 p-3 bg-yellow-50 border-l-4 border-yellow-400 rounded text-sm text-yellow-800">
                Unable to load collections. Showing "All Collections" only.
              </div>
            )}

            {isLoading && <LoadingSpinner message="Searching..." />}

            {!isLoading && showResults && <SearchResults results={results} />}

            {!isLoading && showNoResults && (
              <EmptyState type="no-results" query={query} />
            )}

            {!isLoading && showEmptyState && (
              <EmptyState type="no-query" />
            )}
          </div>
        </div>
      </div>
    </ErrorBoundary>
  );
}
