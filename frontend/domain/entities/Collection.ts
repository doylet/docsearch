/**
 * Collection Entity
 *
 * Represents a named collection of documents for organization and filtering.
 * Collections enable users to group related documents and filter search results.
 */
export interface Collection {
  /** Unique name identifier for the collection */
  name: string;

  /** Human-readable description of the collection's purpose */
  description?: string;

  /** Number of documents currently in this collection */
  document_count: number;

  /** Creation timestamp (ISO 8601 format) */
  created_at: string;
}

/**
 * Type guard to check if an object is a valid Collection
 */
export function isCollection(obj: unknown): obj is Collection {
  const coll = obj as Collection;
  return (
    typeof coll?.name === 'string' &&
    (coll?.description === undefined || typeof coll.description === 'string') &&
    typeof coll?.document_count === 'number' &&
    typeof coll?.created_at === 'string'
  );
}
