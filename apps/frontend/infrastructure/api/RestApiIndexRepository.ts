import { IndexRepository, IndexOptions } from '@/domain/repositories/IndexRepository';
import { IndexOperation, IndexOperationStatus } from '@/domain/entities/IndexOperation';
import { RestApiClient } from './RestApiClient';
import { buildApiUrl, API_ENDPOINTS } from '../config/apiConfig';

/**
 * RestApiIndexRepository
 *
 * Infrastructure implementation of IndexRepository using REST API.
 * Adapts backend API responses to domain entities.
 * Follows Liskov Substitution Principle - can be swapped with any IndexRepository implementation.
 */
export class RestApiIndexRepository implements IndexRepository {
  constructor(private readonly apiClient: RestApiClient) {}

  async indexPath(path: string, options?: IndexOptions): Promise<IndexOperation> {
    const requestBody: IndexPathRequest = {
      path,
      collection: options?.collection || 'default',
      recursive: options?.recursive ?? true,
      include_patterns: options?.includePatterns,
      exclude_patterns: options?.excludePatterns,
    };

    const url = buildApiUrl(API_ENDPOINTS.index);
    const response = await this.apiClient.post<ApiIndexResponse>(url, requestBody);

    return this.mapApiOperationToDomain(response);
  }

  async indexFile(path: string, options?: IndexOptions): Promise<IndexOperation> {
    const requestBody: IndexFileRequest = {
      path,
      collection: options?.collection || 'default',
    };

    const url = buildApiUrl(API_ENDPOINTS.index);
    const response = await this.apiClient.post<ApiIndexResponse>(url, requestBody);

    return this.mapApiOperationToDomain(response);
  }

  async getStatus(operationId: string): Promise<IndexOperation> {
    const url = buildApiUrl(API_ENDPOINTS.indexStatus(operationId));
    const response = await this.apiClient.get<ApiIndexResponse>(url);

    return this.mapApiOperationToDomain(response);
  }

  /**
   * Map API index operation format to domain IndexOperation entity
   */
  private mapApiOperationToDomain(apiOp: ApiIndexResponse): IndexOperation {
    return {
      id: apiOp.id || apiOp.operation_id || 'unknown',
      path: apiOp.path,
      collection: apiOp.collection,
      status: this.mapApiStatusToDomain(apiOp.status),
      documents_processed: apiOp.documents_processed || 0,
      error_count: apiOp.error_count || 0,
      errors: apiOp.errors,
      started_at: apiOp.started_at,
      completed_at: apiOp.completed_at,
      progress: apiOp.progress,
    };
  }

  /**
   * Map API status string to domain IndexOperationStatus
   */
  private mapApiStatusToDomain(apiStatus: string): IndexOperationStatus {
    const statusMap: Record<string, IndexOperationStatus> = {
      'pending': 'pending',
      'in_progress': 'in_progress',
      'running': 'in_progress', // Backend might use 'running'
      'completed': 'completed',
      'success': 'completed',  // Backend might use 'success'
      'failed': 'failed',
      'error': 'failed',       // Backend might use 'error'
      'cancelled': 'cancelled',
    };

    return statusMap[apiStatus.toLowerCase()] || 'failed';
  }
}

/**
 * API Request/Response Types
 * These match the backend REST API contract.
 * Defined here (infrastructure layer) not in domain layer.
 */

interface IndexPathRequest {
  path: string;
  collection: string;
  recursive: boolean;
  include_patterns?: string[];
  exclude_patterns?: string[];
}

interface IndexFileRequest {
  path: string;
  collection: string;
}

interface ApiIndexResponse {
  id?: string;
  operation_id?: string;  // Backend might use operation_id
  path: string;
  collection: string;
  status: string;
  documents_processed?: number;
  error_count?: number;
  errors?: string[];
  started_at?: string;
  completed_at?: string;
  progress?: number;
}
