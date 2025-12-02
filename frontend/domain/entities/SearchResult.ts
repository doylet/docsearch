import { Document } from './Document';

/**
 * SearchResult Entity
 *
 * Aggregate root representing a single search result with document and metadata.
 * Contains the matched document plus search-specific information like highlights.
 */
export interface SearchResult {
  /** The document that matched the search query */
  document: Document;

  /** Relevance score (0.0 to 1.0, higher is more relevant) */
  score: number;

  /** Text snippets showing query matches in context */
  highlights?: string[];

  /** Ranking position in the result set (1-based) */
  rank?: number;
}

/**
 * Type guard to check if an object is a valid SearchResult
 */
export function isSearchResult(obj: unknown): obj is SearchResult {
  const result = obj as SearchResult;
  return (
    result?.document !== undefined &&
    typeof result?.score === 'number' &&
    (result?.highlights === undefined || Array.isArray(result.highlights)) &&
    (result?.rank === undefined || typeof result.rank === 'number')
  );
}
