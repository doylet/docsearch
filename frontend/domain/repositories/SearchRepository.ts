import { SearchResult } from '../entities/SearchResult';

/**
 * Search Options
 *
 * Configuration parameters for search operations.
 */
export interface SearchOptions {
  /** Optional collection name to filter results */
  collection?: string;

  /** Maximum number of results to return (default: backend determines) */
  limit?: number;

  /** Include content snippets in results */
  includeSnippets?: boolean;
}

/**
 * SearchRepository Interface
 *
 * Defines the contract for searching documents.
 * Infrastructure layer implements this interface to communicate with the backend API.
 * Follows Interface Segregation Principle - only search operations.
 */
export interface SearchRepository {
  /**
   * Search for documents matching the query
   *
   * @param query - Natural language search query (minimum 2 characters)
   * @param options - Search configuration options
   * @returns Promise resolving to array of search results ordered by relevance
   * @throws Error if query is invalid or backend communication fails
   */
  search(query: string, options?: SearchOptions): Promise<SearchResult[]>;
}
