import { Collection } from '../entities/Collection';

/**
 * CollectionRepository Interface
 *
 * Defines the contract for managing document collections.
 * Infrastructure layer implements this interface to communicate with the backend API.
 * Follows Interface Segregation Principle - only collection operations.
 */
export interface CollectionRepository {
  /**
   * Get all available collections
   *
   * @returns Promise resolving to array of collections
   * @throws Error if backend communication fails
   */
  list(): Promise<Collection[]>;

  /**
   * Get a specific collection by name
   *
   * @param name - Collection name
   * @returns Promise resolving to the collection
   * @throws Error if collection not found or backend communication fails
   */
  get(name: string): Promise<Collection>;

  /**
   * Create a new collection
   *
   * @param name - Collection name (must be unique)
   * @param description - Optional description
   * @returns Promise resolving to the created collection
   * @throws Error if collection already exists or backend communication fails
   */
  create(name: string, description?: string): Promise<Collection>;

  /**
   * Delete a collection and all its documents
   *
   * @param name - Collection name
   * @returns Promise resolving when deletion completes
   * @throws Error if collection not found or backend communication fails
   */
  delete(name: string): Promise<void>;
}
