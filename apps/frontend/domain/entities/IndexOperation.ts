/**
 * IndexOperation Entity
 *
 * Represents an ongoing or completed indexing operation.
 * Tracks progress, errors, and final statistics for file/directory indexing.
 */

/** Possible states for an indexing operation */
export type IndexOperationStatus =
  | 'pending'      // Queued but not started
  | 'in_progress'  // Currently processing
  | 'completed'    // Successfully finished
  | 'failed'       // Terminated with errors
  | 'cancelled';   // User cancelled

export interface IndexOperation {
  /** Unique identifier for this operation */
  id: string;

  /** Filesystem path being indexed (file or directory) */
  path: string;

  /** Target collection name */
  collection: string;

  /** Current operation status */
  status: IndexOperationStatus;

  /** Number of documents successfully processed */
  documents_processed: number;

  /** Number of documents that failed to index */
  error_count: number;

  /** Array of error messages for failed files */
  errors?: string[];

  /** Start timestamp (ISO 8601 format) */
  started_at?: string;

  /** Completion timestamp (ISO 8601 format) */
  completed_at?: string;

  /** Progress percentage (0-100) */
  progress?: number;
}

/**
 * Type guard to check if an object is a valid IndexOperation
 */
export function isIndexOperation(obj: unknown): obj is IndexOperation {
  const op = obj as IndexOperation;
  return (
    typeof op?.id === 'string' &&
    typeof op?.path === 'string' &&
    typeof op?.collection === 'string' &&
    typeof op?.status === 'string' &&
    typeof op?.documents_processed === 'number' &&
    typeof op?.error_count === 'number' &&
    (op?.errors === undefined || Array.isArray(op.errors)) &&
    (op?.started_at === undefined || typeof op.started_at === 'string') &&
    (op?.completed_at === undefined || typeof op.completed_at === 'string') &&
    (op?.progress === undefined || typeof op.progress === 'number')
  );
}
