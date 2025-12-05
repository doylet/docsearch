import { IndexOperation } from '../entities/IndexOperation';

/**
 * Index Options
 *
 * Configuration parameters for indexing operations.
 */
export interface IndexOptions {
  /** Target collection name (default: "default") */
  collection?: string;

  /** Whether to recursively index subdirectories (for directory indexing) */
  recursive?: boolean;

  /** File patterns to include (e.g., ["*.md", "*.txt"]) */
  includePatterns?: string[];

  /** File patterns to exclude (e.g., ["node_modules/**"]) */
  excludePatterns?: string[];
}

/**
 * IndexRepository Interface
 *
 * Defines the contract for indexing files and directories.
 * Infrastructure layer implements this interface to communicate with the backend API.
 * Follows Interface Segregation Principle - only indexing operations.
 */
export interface IndexRepository {
  /**
   * Index a directory and all eligible files within it
   *
   * @param path - Filesystem path to directory
   * @param options - Indexing configuration options
   * @returns Promise resolving to IndexOperation with progress tracking
   * @throws Error if path is invalid or backend communication fails
   */
  indexPath(path: string, options?: IndexOptions): Promise<IndexOperation>;

  /**
   * Index a single file
   *
   * @param path - Filesystem path to file
   * @param options - Indexing configuration options
   * @returns Promise resolving to IndexOperation result
   * @throws Error if path is invalid or backend communication fails
   */
  indexFile(path: string, options?: IndexOptions): Promise<IndexOperation>;

  /**
   * Get status of an ongoing indexing operation
   *
   * @param operationId - Unique identifier for the operation
   * @returns Promise resolving to current IndexOperation state
   * @throws Error if operation not found or backend communication fails
   */
  getStatus(operationId: string): Promise<IndexOperation>;
}
