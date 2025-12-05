/**
 * Document Entity
 *
 * Represents a searchable document in the system with metadata and content.
 * Corresponds to backend Document model.
 */
export interface Document {
  /** Unique identifier for the document */
  document_id: string;

  /** Human-readable title of the document */
  title: string;

  /** Filesystem path to the document */
  path: string;

  /** Extracted text content (may be truncated for display) */
  content: string;

  /** Relevance score from search (0.0 to 1.0, higher is more relevant) */
  score?: number;

  /** Collection this document belongs to */
  collection: string;

  /** Last indexed timestamp (ISO 8601 format) */
  indexed_at?: string;
}

/**
 * Type guard to check if an object is a valid Document
 */
export function isDocument(obj: unknown): obj is Document {
  const doc = obj as Document;
  return (
    typeof doc?.document_id === 'string' &&
    typeof doc?.title === 'string' &&
    typeof doc?.path === 'string' &&
    typeof doc?.content === 'string' &&
    typeof doc?.collection === 'string' &&
    (doc?.score === undefined || typeof doc.score === 'number') &&
    (doc?.indexed_at === undefined || typeof doc.indexed_at === 'string')
  );
}
