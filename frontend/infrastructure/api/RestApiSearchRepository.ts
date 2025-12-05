import { SearchRepository, SearchOptions } from '@/domain/repositories/SearchRepository';
import { SearchResult } from '@/domain/entities/SearchResult';
import { Document } from '@/domain/entities/Document';
import { RestApiClient } from './RestApiClient';
import { buildApiUrl, API_ENDPOINTS } from '../config/apiConfig';

/**
 * RestApiSearchRepository
 *
 * Infrastructure implementation of SearchRepository using REST API.
 * Adapts backend API responses to domain entities.
 * Follows Liskov Substitution Principle - can be swapped with any SearchRepository implementation.
 */
export class RestApiSearchRepository implements SearchRepository {
  constructor(private readonly apiClient: RestApiClient) {}

  async search(query: string, options?: SearchOptions): Promise<SearchResult[]> {
    // Build request body
    const body: Record<string, string | number> = {
      query: query,
      collection: options?.collection || 'zero_latency_docs',
    };

    if (options?.limit) {
      body.limit = options.limit;
    }

    // Make API request
    const url = buildApiUrl(API_ENDPOINTS.search);
    const response = await this.apiClient.post<ApiSearchResponse>(url, body);

    // Transform API response to domain entities
    return response.results.map((apiResult, index) => ({
      document: this.mapApiDocumentToDomain(apiResult.document),
      score: apiResult.score,
      highlights: apiResult.highlights,
      rank: index + 1,
    }));
  }

  /**
   * Map API document format to domain Document entity
   */
  private mapApiDocumentToDomain(apiDoc: ApiDocument): Document {
    return {
      document_id: apiDoc.document_id,
      title: apiDoc.title,
      path: apiDoc.path,
      content: apiDoc.content || '',
      score: apiDoc.score,
      collection: apiDoc.collection,
      indexed_at: apiDoc.indexed_at,
    };
  }
}

/**
 * API Response Types
 * These match the backend REST API contract.
 * Defined here (infrastructure layer) not in domain layer.
 */

interface ApiDocument {
  document_id: string;
  title: string;
  path: string;
  content?: string;
  score?: number;
  collection: string;
  indexed_at?: string;
}

interface ApiSearchResult {
  document: ApiDocument;
  score: number;
  highlights?: string[];
}

interface ApiSearchResponse {
  results: ApiSearchResult[];
  total: number;
  query: string;
}
