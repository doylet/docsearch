'use client';

import { Collection } from '@/domain/entities/Collection';

interface CollectionSelectorProps {
  collections: Collection[] | undefined;
  selectedCollection: string;
  onCollectionChange: (collection: string) => void;
  isLoading?: boolean;
}

/**
 * CollectionSelector Component
 *
 * Dropdown selector for filtering search results by collection.
 * Part of User Story 2 (P2) - Collection-Filtered Search.
 *
 * Features:
 * - "All Collections" option to search across all collections
 * - Individual collection options with document counts
 * - Accessible with aria-label for screen readers
 * - Graceful handling of loading states
 */
export function CollectionSelector({
  collections,
  selectedCollection,
  onCollectionChange,
  isLoading = false
}: CollectionSelectorProps) {
  return (
    <div className="flex items-center gap-2">
      <label htmlFor="collection-selector" className="text-sm font-medium text-gray-700">
        Filter by:
      </label>
      <select
        id="collection-selector"
        value={selectedCollection}
        onChange={(e) => onCollectionChange(e.target.value)}
        disabled={isLoading}
        className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white disabled:bg-gray-100 disabled:cursor-not-allowed transition-colors"
        aria-label="Filter search results by collection"
      >
        <option value="all">All Collections</option>
        {collections?.map((collection) => (
          <option key={collection.name} value={collection.name}>
            {collection.name} ({collection.document_count} doc{collection.document_count !== 1 ? 's' : ''})
          </option>
        ))}
      </select>
    </div>
  );
}
