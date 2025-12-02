import { SearchRepository, SearchOptions } from '../repositories/SearchRepository';
import { SearchResult } from '../entities/SearchResult';

/**
 * SearchDocumentsUseCase
 *
 * Application-layer use case for searching documents.
 * Orchestrates search operations and applies business rules.
 * Follows Single Responsibility Principle - only handles search logic.
 */
export class SearchDocumentsUseCase {
  constructor(private readonly searchRepository: SearchRepository) {}

  /**
   * Execute a search query
   *
   * @param query - Natural language search query
   * @param options - Search configuration options
   * @returns Promise resolving to array of search results
   * @throws Error if query is invalid (less than 2 characters)
   */
  async search(query: string, options?: SearchOptions): Promise<SearchResult[]> {
    // Business rule: minimum query length validation
    if (query.trim().length < 2) {
      throw new Error('Search query must be at least 2 characters long');
    }

    // Delegate to repository (infrastructure layer handles API communication)
    const results = await this.searchRepository.search(query.trim(), options);

    // Business rule: add ranking to results if not present
    return results.map((result, index) => ({
      ...result,
      rank: result.rank ?? index + 1,
    }));
  }

  /**
   * Check if a query is valid for searching
   *
   * @param query - Query string to validate
   * @returns true if query meets minimum requirements
   */
  isValidQuery(query: string): boolean {
    return query.trim().length >= 2;
  }
}
