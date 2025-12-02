import { IndexRepository, IndexOptions } from '../repositories/IndexRepository';
import { IndexOperation } from '../entities/IndexOperation';

/**
 * IndexDocumentsUseCase
 *
 * Application-layer use case for indexing files and directories.
 * Orchestrates indexing operations and applies business rules.
 * Follows Single Responsibility Principle - only handles indexing logic.
 */
export class IndexDocumentsUseCase {
  constructor(private readonly indexRepository: IndexRepository) {}

  /**
   * Index a directory and all eligible files within it
   *
   * @param path - Filesystem path to directory
   * @param options - Indexing configuration options
   * @returns Promise resolving to IndexOperation with tracking info
   * @throws Error if path is empty or invalid
   */
  async indexPath(path: string, options?: IndexOptions): Promise<IndexOperation> {
    // Business rule: path validation
    if (!path || path.trim().length === 0) {
      throw new Error('Path cannot be empty');
    }

    // Business rule: default collection name if not provided
    const indexOptions: IndexOptions = {
      ...options,
      collection: options?.collection || 'default',
      recursive: options?.recursive ?? true, // Default to recursive
    };

    return await this.indexRepository.indexPath(path.trim(), indexOptions);
  }

  /**
   * Index a single file
   *
   * @param path - Filesystem path to file
   * @param options - Indexing configuration options
   * @returns Promise resolving to IndexOperation result
   * @throws Error if path is empty or invalid
   */
  async indexFile(path: string, options?: IndexOptions): Promise<IndexOperation> {
    // Business rule: path validation
    if (!path || path.trim().length === 0) {
      throw new Error('Path cannot be empty');
    }

    // Business rule: default collection name if not provided
    const indexOptions: IndexOptions = {
      ...options,
      collection: options?.collection || 'default',
    };

    return await this.indexRepository.indexFile(path.trim(), indexOptions);
  }

  /**
   * Get the current status of an indexing operation
   *
   * @param operationId - Unique identifier for the operation
   * @returns Promise resolving to current operation state
   */
  async getStatus(operationId: string): Promise<IndexOperation> {
    if (!operationId || operationId.trim().length === 0) {
      throw new Error('Operation ID cannot be empty');
    }

    return await this.indexRepository.getStatus(operationId);
  }

  /**
   * Check if a path is valid for indexing
   *
   * @param path - Path string to validate
   * @returns true if path meets minimum requirements
   */
  isValidPath(path: string): boolean {
    return path !== null && path !== undefined && path.trim().length > 0;
  }
}
