import { CollectionRepository } from '@/domain/repositories/CollectionRepository';
import { Collection } from '@/domain/entities/Collection';
import { RestApiClient } from './RestApiClient';
import { buildApiUrl, API_ENDPOINTS } from '../config/apiConfig';

/**
 * RestApiCollectionRepository
 *
 * Infrastructure implementation of CollectionRepository using REST API.
 * Adapts backend API responses to domain entities.
 * Follows Liskov Substitution Principle - can be swapped with any CollectionRepository implementation.
 */
export class RestApiCollectionRepository implements CollectionRepository {
  constructor(private readonly apiClient: RestApiClient) {}

  async list(): Promise<Collection[]> {
    const url = buildApiUrl(API_ENDPOINTS.collections);
    const response = await this.apiClient.get<ApiCollectionsResponse>(url);

    return response.collections.map(this.mapApiCollectionToDomain);
  }

  async get(name: string): Promise<Collection> {
    const url = buildApiUrl(`${API_ENDPOINTS.collections}/${encodeURIComponent(name)}`);
    const response = await this.apiClient.get<ApiCollectionResponse>(url);

    return this.mapApiCollectionToDomain(response.collection);
  }

  async create(name: string, description?: string): Promise<Collection> {
    const url = buildApiUrl(API_ENDPOINTS.collections);
    const requestBody: CreateCollectionRequest = {
      name,
      description,
    };

    const response = await this.apiClient.post<ApiCollectionResponse>(url, requestBody);
    return this.mapApiCollectionToDomain(response.collection);
  }

  async delete(name: string): Promise<void> {
    const url = buildApiUrl(`${API_ENDPOINTS.collections}/${encodeURIComponent(name)}`);
    await this.apiClient.delete<void>(url);
  }

  /**
   * Map API collection format to domain Collection entity
   */
  private mapApiCollectionToDomain(apiColl: ApiCollection): Collection {
    return {
      name: apiColl.name,
      description: apiColl.description,
      document_count: apiColl.document_count || apiColl.documentCount || 0,
      created_at: apiColl.created_at || apiColl.createdAt || new Date().toISOString(),
    };
  }
}

/**
 * API Request/Response Types
 * These match the backend REST API contract.
 * Defined here (infrastructure layer) not in domain layer.
 */

interface ApiCollection {
  name: string;
  description?: string;
  document_count?: number;
  documentCount?: number;  // Backend might use camelCase
  created_at?: string;
  createdAt?: string;      // Backend might use camelCase
}

interface ApiCollectionsResponse {
  collections: ApiCollection[];
}

interface ApiCollectionResponse {
  collection: ApiCollection;
}

interface CreateCollectionRequest {
  name: string;
  description?: string;
}
