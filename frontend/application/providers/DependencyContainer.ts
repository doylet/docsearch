'use client';

import { create } from 'zustand';
import { SearchDocumentsUseCase } from '@/domain/usecases/SearchDocumentsUseCase';
import { IndexDocumentsUseCase } from '@/domain/usecases/IndexDocumentsUseCase';
import { ManageCollectionsUseCase } from '@/domain/usecases/ManageCollectionsUseCase';
import { RestApiClient } from '@/infrastructure/api/RestApiClient';
import { RestApiSearchRepository } from '@/infrastructure/api/RestApiSearchRepository';
import { RestApiIndexRepository } from '@/infrastructure/api/RestApiIndexRepository';
import { RestApiCollectionRepository } from '@/infrastructure/api/RestApiCollectionRepository';

/**
 * Dependency Container State
 *
 * Holds singleton instances of use cases for dependency injection.
 * Follows Dependency Inversion Principle - provides abstractions to application layer.
 */
interface DependencyContainerState {
  searchUseCase: SearchDocumentsUseCase;
  indexUseCase: IndexDocumentsUseCase;
  collectionsUseCase: ManageCollectionsUseCase;
}

/**
 * Initialize dependencies
 *
 * Wires up the entire dependency graph:
 * API Client -> Repositories -> Use Cases
 */
function createDependencies(): DependencyContainerState {
  // Infrastructure layer: API client
  const apiClient = new RestApiClient();

  // Infrastructure layer: Repository implementations
  const searchRepository = new RestApiSearchRepository(apiClient);
  const indexRepository = new RestApiIndexRepository(apiClient);
  const collectionRepository = new RestApiCollectionRepository(apiClient);

  // Domain layer: Use cases (inject repository dependencies)
  const searchUseCase = new SearchDocumentsUseCase(searchRepository);
  const indexUseCase = new IndexDocumentsUseCase(indexRepository);
  const collectionsUseCase = new ManageCollectionsUseCase(collectionRepository);

  return {
    searchUseCase,
    indexUseCase,
    collectionsUseCase,
  };
}

/**
 * Dependency Container Store
 *
 * Zustand store providing singleton access to use cases.
 * Used by React hooks via useDependencyContainer().
 */
export const useDependencyContainer = create<DependencyContainerState>(() => createDependencies());
