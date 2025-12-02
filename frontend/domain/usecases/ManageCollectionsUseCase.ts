import { CollectionRepository } from '../repositories/CollectionRepository';
import { Collection } from '../entities/Collection';

/**
 * ManageCollectionsUseCase
 *
 * Application-layer use case for managing document collections.
 * Orchestrates collection operations and applies business rules.
 * Follows Single Responsibility Principle - only handles collection management.
 */
export class ManageCollectionsUseCase {
  constructor(private readonly collectionRepository: CollectionRepository) {}

  /**
   * Get all available collections
   *
   * @returns Promise resolving to array of collections sorted by name
   */
  async listCollections(): Promise<Collection[]> {
    const collections = await this.collectionRepository.list();

    // Business rule: always sort collections alphabetically for consistent UX
    return collections.sort((a, b) => a.name.localeCompare(b.name));
  }

  /**
   * Get a specific collection by name
   *
   * @param name - Collection name
   * @returns Promise resolving to the collection
   * @throws Error if name is invalid or collection not found
   */
  async getCollection(name: string): Promise<Collection> {
    if (!name || name.trim().length === 0) {
      throw new Error('Collection name cannot be empty');
    }

    return await this.collectionRepository.get(name.trim());
  }

  /**
   * Create a new collection
   *
   * @param name - Collection name (must be unique and valid)
   * @param description - Optional description
   * @returns Promise resolving to the created collection
   * @throws Error if name is invalid or collection already exists
   */
  async createCollection(name: string, description?: string): Promise<Collection> {
    // Business rule: collection name validation
    if (!name || name.trim().length === 0) {
      throw new Error('Collection name cannot be empty');
    }

    // Business rule: name format validation (alphanumeric, hyphen, underscore only)
    const namePattern = /^[a-zA-Z0-9_-]+$/;
    if (!namePattern.test(name.trim())) {
      throw new Error('Collection name must contain only letters, numbers, hyphens, and underscores');
    }

    return await this.collectionRepository.create(name.trim(), description?.trim());
  }

  /**
   * Delete a collection and all its documents
   *
   * @param name - Collection name
   * @returns Promise resolving when deletion completes
   * @throws Error if name is invalid or collection not found
   */
  async deleteCollection(name: string): Promise<void> {
    if (!name || name.trim().length === 0) {
      throw new Error('Collection name cannot be empty');
    }

    // Business rule: prevent deletion of default collection
    if (name.trim().toLowerCase() === 'default') {
      throw new Error('Cannot delete the default collection');
    }

    await this.collectionRepository.delete(name.trim());
  }

  /**
   * Check if a collection name is valid
   *
   * @param name - Collection name to validate
   * @returns true if name meets all requirements
   */
  isValidCollectionName(name: string): boolean {
    if (!name || name.trim().length === 0) {
      return false;
    }

    const namePattern = /^[a-zA-Z0-9_-]+$/;
    return namePattern.test(name.trim());
  }
}
