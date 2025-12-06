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
      document: this.mapApiDocumentToDomain(apiResult),
      score: apiResult.final_score,
      highlights: [], // highlights not in current API response
      rank: index + 1,
    }));
  }

  /**
   * Map API document format to domain Document entity
   */
  private mapApiDocumentToDomain(apiDoc: ApiSearchResult): Document {
    return {
      document_id: apiDoc.document_id,
      title: apiDoc.title,
      path: apiDoc.document_path,
      content: apiDoc.content || '',
      score: apiDoc.final_score,
      collection: apiDoc.collection,
      indexed_at: undefined, // not in current API response
    };
  }
}

/**
 * API Response Types
 * These match the backend REST API contract.
 * Defined here (infrastructure layer) not in domain layer.
 */

interface ApiSearchResult {
  doc_id: {
    collection: string;
    external_id: string;
    version: number;
  };
  chunk_id: string;
  document_id: string;
  uri: string;
  title: string;
  document_path: string;
  content: string;
  snippet: string;
  section_path: string[];
  heading_path: string[];
  scores: {
    bm25_raw: number | null;
    vector_raw: number;
    bm25_normalized: number | null;
    vector_normalized: number;
    fused: number;
    normalization_method: string;
  };
  final_score: number;
  from_signals: {
    bm25: boolean;
    vector: boolean;
    variants: number[];
    query_expansion: boolean;
  };
  ranking_signals: any;
  url: string | null;
  collection: string;
  custom_metadata: {
    parent_document_id: string;
    collection: string;
    chunk_index: string;
  };
}

interface ApiSearchResponse {
  results: ApiSearchResult[];
  total_count: number | null;
  search_metadata: {
    query: {
      raw: string;
      normalized: string;
      enhanced: string;
      limit: number;
    };
    execution_time: {
      secs: number;
      nanos: number;
    };
    query_enhancement_applied: boolean;
    ranking_method: string;
    result_sources: string[];
    debug_info: any;
  };
  pagination: any;
}
